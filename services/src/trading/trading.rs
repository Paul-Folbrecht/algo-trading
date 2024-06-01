use std::sync::Arc;

use crate::historical_data::*;
use crate::market_data::*;
use crate::strategy::*;
use chrono::NaiveDate;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new<'market_data>(
    strategy_name: String,
    symbols: &'market_data Vec<String>,
    market_data_service: Arc<impl MarketDataService + 'market_data + Send + Sync>,
    historical_data_service: Arc<impl HistoricalDataService + 'market_data + Send + Sync>,
) -> impl TradingService + 'market_data {
    let strategy = crate::strategy::Strategy::new(strategy_name, symbols.clone());
    Trading {
        strategy,
        symbols,
        market_data_service,
        historical_data_service,
        thread_handle: None,
    }
}

pub struct Trading<
    'market_data,
    M: MarketDataService + Send + Sync,
    H: HistoricalDataService + Send + Sync,
> {
    strategy: Strategy,
    symbols: &'market_data Vec<String>,
    market_data_service: Arc<M>,
    historical_data_service: Arc<H>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

mod implementation {
    use super::*;
    use domain::domain::SymbolData;
    use std::collections::HashMap;

    impl<
            'market_data,
            M: MarketDataService + Send + Sync,
            H: HistoricalDataService + Send + Sync,
        > TradingService for Trading<'market_data, M, H>
    {
        fn run(&mut self) -> Result<(), String> {
            println!("Running TradingService with strategy: {:?}", self.strategy);
            let symbol_data = load_history(self.symbols, self.historical_data_service.clone());

            match self.market_data_service.subscribe() {
                Ok(rx) => {
                    println!("TradingService subscribed to MarketDataService");
                    let strategy = self.strategy.clone();
                    self.thread_handle = Some(std::thread::spawn(move || loop {
                        match rx.recv() {
                            Ok(quote) => {
                                println!("TradingService received quote:\n{:?}", quote);
                                strategy.handle(&quote, symbol_data.get(&quote.symbol).unwrap());
                            }
                            Err(e) => {
                                println!("Error on receive!: {}", e);
                            }
                        }
                    }));
                }

                Err(e) => return Err(format!("Failed to subscribe to MarketDataService: {}", e)),
            }
            Ok(())
        }
    }

    pub fn load_history<'market_data>(
        symbols: &'market_data Vec<String>,
        historical_data_service: Arc<impl HistoricalDataService + 'market_data>,
    ) -> HashMap<String, SymbolData> {
        symbols
            .iter()
            .map(|symbol| -> (String, SymbolData) {
                let start_date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
                let end_date = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();
                let query: Result<Vec<domain::domain::Day>, reqwest::Error> =
                    historical_data_service
                        .fetch(symbol, start_date, end_date)
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
                        (symbol.to_owned(), data)
                    }
                    Err(e) => panic!("Can't load history for {}: {}", symbol, e),
                }
            })
            .into_iter()
            .collect()
    }
}

mod test {
    use std::{collections::HashMap, sync::Arc};

    pub trait Service1 {
        fn get_data(&self) -> HashMap<String, f64>;
    }

    pub trait Service {
        fn run(&mut self) -> ();
    }

    pub struct ServiceStruct<'data, M: Service1 + Send + Sync> {
        names: &'data Vec<String>,
        service_1: Arc<M>,
    }

    impl<'data, M: Service1 + Send + Sync> Service for ServiceStruct<'data, M> {
        fn run(&mut self) -> () {
            let data = load(self.names, self.service_1.clone());
            let (tx, rx) = std::sync::mpsc::channel::<String>();

            std::thread::spawn(move || loop {
                match rx.recv() {
                    Ok(q) => {
                        println!("{:?}", q);
                    }
                    Err(e) => {
                        println!("Error on receive!: {}", e);
                    }
                }
            });
        }
    }

    pub fn load<'data>(
        names: &'data Vec<String>,
        service1: Arc<impl Service1 + 'data>,
    ) -> HashMap<String, f64> {
        service1.get_data()
    }
}

#[cfg(test)]
#[path = "./../tests/trading_test.rs"]
mod trading_test;
