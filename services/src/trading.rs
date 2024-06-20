use std::sync::Arc;

use crate::historical_data::HistoricalDataService;
use crate::market_data::MarketDataService;
use crate::orders::OrderService;
use domain::domain::*;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new(
    strategy_name: String,
    symbols: Vec<String>,
    market_data: Arc<impl MarketDataService + 'static + Send + Sync>,
    historical_data: Arc<impl HistoricalDataService + 'static + Send + Sync>,
    orders: Arc<impl OrderService + 'static + Send + Sync>,
) -> impl TradingService + 'static {
    implementation::Trading {
        strategy_name,
        symbols,
        market_data,
        historical_data,
        orders,
        thread_handle: None,
    }
}

mod implementation {
    use super::*;
    use crate::orders::OrderService;
    use chrono::Local;
    use domain::domain::SymbolData;
    use std::collections::HashMap;

    pub struct Trading<
        M: MarketDataService + 'static + Send + Sync,
        H: HistoricalDataService + 'static + Send + Sync,
        O: OrderService + 'static + Send + Sync,
    > {
        pub strategy_name: String,
        pub symbols: Vec<String>,
        pub market_data: Arc<M>,
        pub historical_data: Arc<H>,
        pub orders: Arc<O>,
        pub thread_handle: Option<std::thread::JoinHandle<()>>,
    }

    impl<
            M: MarketDataService + Send + Sync,
            H: HistoricalDataService + Send + Sync,
            O: OrderService + Send + Sync,
        > TradingService for Trading<M, H, O>
    {
        fn run(&mut self) -> Result<(), String> {
            println!(
                "Running TradingService with strategy: {:?}",
                self.strategy_name
            );
            let symbol_data = load_history(&self.symbols, self.historical_data.clone());
            let orders: Arc<O> = self.orders.clone();

            match self.market_data.subscribe() {
                Ok(rx) => {
                    println!("TradingService subscribed to MarketDataService");
                    let strategy = Strategy::new(&self.strategy_name, self.symbols.clone());

                    self.thread_handle = Some(std::thread::spawn(move || loop {
                        handle_quote(&rx, &orders);
                    }));
                }

                Err(e) => return Err(format!("Failed to subscribe to MarketDataService: {}", e)),
            }
            Ok(())
        }
    }

    fn handle_quote<O: OrderService + Send + Sync>(
        rx: &crossbeam_channel::Receiver<Quote>,
        orders: &Arc<O>,
    ) {
        match rx.recv() {
            Ok(quote) => {
                let order = Order {
                    symbol: quote.symbol.clone(),
                    qty: 1,
                    date: Local::now().naive_local().date(),
                    side: Side::Buy,
                    tradier_id: None,
                };
                orders.create_order(order);
            }
            Err(e) => {
                eprintln!("Error on receive!: {}", e);
            }
        }
    }

    // fn handle_quote<'static, O: OrderService + Send + Sync + 'static>(
    //     symbol_data: &HashMap<String, SymbolData>,
    //     quote: Quote,
    //     strategy: &Strategy,
    //     orders: Arc<O>,
    // ) {
    //     if let Some(symbol_data) = symbol_data.get(&quote.symbol) {
    //         println!("TradingService received quote:\n{:?}", quote);
    //         let signal = strategy.handle(&quote, symbol_data);
    //         match signal {
    //             Ok(Signal::Buy) => {
    //                 //   - If position qty < target_position_qty, buy the difference
    //             }
    //             Ok(Signal::Sell) => {
    //                 //   - If we have a position, unwind
    //             }
    //             Ok(Signal::None) => {}
    //             Err(e) => {
    //                 eprintln!("Error from strategy: {}", e);
    //             }
    //         }
    //     }
    // }

    // impl<
    //         'static,
    //         M: MarketDataService + Send + Sync,
    //         H: HistoricalDataService + Send + Sync,
    //         O: OrderService + Send + Sync,
    //     > Trading<'static, M, H, O>
    // {
    //     fn handle_quote(
    //         &self,
    //         symbol_data: &HashMap<String, SymbolData>,
    //         quote: Quote,
    //         strategy: &Strategy,
    //         orders: O,
    //     ) {
    //         if let Some(symbol_data) = symbol_data.get(&quote.symbol) {
    //             println!("TradingService received quote:\n{:?}", quote);
    //             let signal = strategy.handle(&quote, symbol_data);
    //             match signal {
    //                 Ok(Signal::Buy) => {
    //                     //   - If position qty < target_position_qty, buy the difference
    //                 }
    //                 Ok(Signal::Sell) => {
    //                     //   - If we have a position, unwind
    //                 }
    //                 Ok(Signal::None) => {}
    //                 Err(e) => {
    //                     eprintln!("Error from strategy: {}", e);
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn load_history(
        symbols: &Vec<String>,
        historical_data_service: Arc<impl HistoricalDataService + 'static>,
    ) -> HashMap<String, SymbolData> {
        symbols
            .iter()
            .map(|symbol| -> (String, SymbolData) {
                let end = Local::now().naive_local().date();
                let start = end - chrono::Duration::days(20);
                println!("Loading history for {} from {} to {}", symbol, start, end);
                let query: Result<Vec<domain::domain::Day>, reqwest::Error> =
                    historical_data_service
                        .fetch(symbol, start, end)
                        .map(|h| h.day);
                match query {
                    Ok(history) => {
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
                            history,
                            mean,
                            std_dev,
                        };
                        println!("Loaded history for {}: {:?}", symbol, data);
                        (symbol.to_owned(), data)
                    }
                    Err(e) => panic!("Can't load history for {}: {}", symbol, e),
                }
            })
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
#[path = "./tests/trading_test.rs"]
mod trading_test;
