use crate::historical_data::HistoricalDataService;
use crate::market_data::MarketDataService;
use crate::orders::OrderService;
use chrono::NaiveDate;
use domain::domain::*;
use log::*;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
    fn shutdown(&mut self) -> Result<(), String>;
}

pub fn new(
    today: NaiveDate, // The date we're trading for - if backtesting, this is not the current date
    strategy_name: String,
    symbols: Vec<String>,
    capital: HashMap<String, i64>,
    market_data: Arc<impl MarketDataService + 'static + Send + Sync>,
    historical_data: Arc<impl HistoricalDataService + 'static + Send + Sync>,
    orders: Arc<impl OrderService + 'static + Send + Sync>,
    shutdown: Arc<AtomicBool>,
) -> impl TradingService + 'static {
    implementation::Trading {
        today,
        strategy_name,
        symbols,
        capital,
        market_data,
        historical_data,
        orders,
        thread_handle: None,
        rx: None,
        shutdown,
    }
}

mod implementation {
    use super::*;
    use crossbeam_channel::{Receiver, TryRecvError};
    use std::{
        collections::HashMap,
        thread::{self, JoinHandle},
        time::Duration,
    };

    pub struct Trading<
        M: MarketDataService + 'static + Send + Sync,
        H: HistoricalDataService + 'static + Send + Sync,
        O: OrderService + 'static + Send + Sync,
    > {
        pub today: NaiveDate,
        pub strategy_name: String,
        pub symbols: Vec<String>,
        pub capital: HashMap<String, i64>,
        pub market_data: Arc<M>,
        pub historical_data: Arc<H>,
        pub orders: Arc<O>,
        pub thread_handle: Option<JoinHandle<()>>,
        pub rx: Option<Receiver<Quote>>,
        pub shutdown: Arc<AtomicBool>,
    }

    impl<
            M: MarketDataService + Send + Sync,
            H: HistoricalDataService + Send + Sync,
            O: OrderService + Send + Sync,
        > TradingService for Trading<M, H, O>
    {
        fn run(&mut self) -> Result<(), String> {
            info!("Running with strategy: {:?}", self.strategy_name);
            let symbol_data: HashMap<String, SymbolData> =
                load_history(self.today, &self.symbols, self.historical_data.clone());
            let orders: Arc<O> = self.orders.clone();

            match self.market_data.subscribe() {
                Ok(rx) => {
                    info!("Subscribed to MarketDataService");
                    let strategy = Strategy::new(&self.strategy_name, self.symbols.clone());
                    let capital = self.capital.clone();
                    let date = self.today;
                    let shutdown = self.shutdown.clone();

                    self.thread_handle = Some(std::thread::spawn(move || {
                        while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                            match rx.try_recv() {
                                Ok(quote) => {
                                    let symbol_capital = capital.get(&quote.symbol).unwrap_or(&0);
                                    info!("Received quote:\n{:?}", quote);
                                    handle_quote(
                                        date,
                                        &symbol_data,
                                        &quote,
                                        *symbol_capital,
                                        &strategy,
                                        orders.clone(),
                                    )
                                }

                                Err(e) => match e {
                                    TryRecvError::Empty => {
                                        thread::sleep(Duration::from_millis(1));
                                    }
                                    TryRecvError::Disconnected => {
                                        info!("MarketData channel disconnected");
                                        break;
                                    }
                                },
                            }
                        }

                        info!("Shutting down");
                    }))
                }
                Err(e) => return Err(format!("Failed to subscribe to MarketDataService: {}", e)),
            }
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), String> {
            self.rx
                .as_ref()
                .map(|rx| (self.market_data.unsubscribe(rx).unwrap()));

            self.thread_handle
                .take()
                .map(|h| h.join().unwrap())
                .map(|_| info!("Shut down"))
                .ok_or("No thread handle to join".to_string())
        }
    }

    pub fn handle_quote(
        date: NaiveDate,
        symbol_data: &HashMap<String, SymbolData>,
        quote: &Quote,
        capital: i64,
        strategy: &Strategy,
        orders: Arc<impl OrderService + 'static>,
    ) {
        if let Some(symbol_data) = symbol_data.get(&quote.symbol) {
            let maybe_position = orders.get_position(&quote.symbol);
            match strategy.handle(quote, symbol_data) {
                Ok(signal) => {
                    if let Some(order) =
                        maybe_create_order(date, signal, maybe_position, quote, capital)
                    {
                        match orders.create_order(order.clone(), strategy.to_string()) {
                            Ok(o) => info!("Order created: {:?}", o),
                            Err(e) => info!("Error creating order: {}", e),
                        }
                    }
                }
                Err(e) => info!("Error from strategy: {}", e),
            }
        } else {
            info!("No symbol data found for {}", quote.symbol);
        }
    }

    pub fn maybe_create_order(
        date: NaiveDate,
        signal: Signal,
        maybe_position: Option<Position>,
        quote: &Quote,
        capital: i64,
    ) -> Option<Order> {
        match signal {
            Signal::Buy => {
                // If position market value < capital, buy up to the limit
                let present_market_value = maybe_position
                    .map(|p| p.quantity as f64 * quote.ask)
                    .unwrap_or(0.0) as i64;
                let remaining_capital = capital - present_market_value;
                let shares = (remaining_capital as f64 / quote.ask) as i64;
                info!(
                    "Buy signal for {} at {}; present_market_value: {}; remaining_capital: {}; shares to buy: {}",
                    quote.symbol, quote.ask, present_market_value, remaining_capital, shares
                );

                match shares {
                    n if n > 0 => Some(Order {
                        symbol: quote.symbol.clone(),
                        quantity: shares,
                        date,
                        side: Side::Buy,
                        id: None,
                        px: Some(quote.ask),
                    }),
                    _ => {
                        info!("Buy signal for {}, but no capital", quote.symbol);
                        None
                    }
                }
            }

            Signal::Sell => {
                // If we have a position, unwind it all
                match maybe_position {
                    Some(p) if p.quantity > 0 => Some(Order {
                        symbol: quote.symbol.clone(),
                        quantity: p.quantity,
                        date,
                        side: Side::Sell,
                        id: None,
                        px: Some(quote.bid),
                    }),
                    _ => {
                        info!(
                            "Sell signal for {}, but no position to unwind",
                            quote.symbol
                        );
                        None
                    }
                }
            }

            Signal::None => None,
        }
    }

    pub fn load_history(
        end: NaiveDate,
        symbols: &[String],
        historical_data_service: Arc<impl HistoricalDataService + 'static>,
    ) -> HashMap<String, SymbolData> {
        let data = historical_data_service.fetch(end);
        symbols
            .iter()
            .map(|symbol| -> (String, SymbolData) {
                let var_name = match data.get(symbol) {
                    Some(history) => {
                        let sum = history.iter().map(|day| day.close).sum::<f64>();
                        let len = history.len() as f64;
                        let mean = sum / len;
                        let variance = history
                            .iter()
                            .map(|quote| (quote.close - mean).powi(2))
                            .sum::<f64>()
                            / len;
                        let std_dev = variance.sqrt();
                        let data = SymbolData {
                            symbol: symbol.clone(),
                            history: history.to_vec(),
                            mean,
                            std_dev,
                        };
                        info!("Initted history for {}", symbol);
                        (symbol.to_owned(), data)
                    }
                    None => panic!("No history for {}", symbol),
                };
                var_name
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "./tests/trading_test.rs"]
mod trading_test;
