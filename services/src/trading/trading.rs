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
    market_data_service: Arc<impl MarketDataService + 'market_data>,
    historical_data_service: Arc<impl HistoricalDataService + 'market_data>,
) -> impl TradingService + 'market_data {
    let strategy = crate::strategy::Strategy::new(strategy_name, symbols);
    Trading {
        strategy,
        symbols,
        market_data_service,
        historical_data_service,
        thread_handle: None,
    }
}

pub struct Trading<'market_data, M: MarketDataService, H: HistoricalDataService> {
    strategy: Strategy<'market_data>,
    symbols: &'market_data Vec<String>,
    market_data_service: Arc<M>,
    historical_data_service: Arc<H>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

mod implementation {
    use std::collections::HashMap;

    use super::*;
    use domain::domain::{Day, Quote};

    pub struct SymbolData {
        pub symbol: String,
        pub history: Vec<domain::domain::Day>,
        pub mean: f64,
        pub std_dev: f64,
    }

    impl<'market_data, M: MarketDataService, H: HistoricalDataService> TradingService
        for Trading<'market_data, M, H>
    {
        fn run(&mut self) -> Result<(), String> {
            println!("Running TradingService with strategy: {:?}", self.strategy);
            load_history(self.symbols, self.historical_data_service.clone());

            match self.market_data_service.subscribe() {
                Ok(rx) => {
                    println!("Subscribed to MarketDataService");
                    self.thread_handle = Some(std::thread::spawn(move || loop {
                        match rx.recv() {
                            Ok(quote) => {
                                println!("TradingService received quote:\n{:?}", quote);
                                handle(quote);
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

    fn handle(quote: Quote) {
        println!("Handling quote: {:?}", quote);
    }

    pub fn load_history<'market_data>(
        symbols: &'market_data Vec<String>,
        historical_data_service: Arc<impl HistoricalDataService + 'market_data>,
    ) -> HashMap<&String, SymbolData> {
        symbols
            .iter()
            .map(|symbol| -> (&String, SymbolData) {
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
                        (symbol, data)
                    }
                    Err(e) => panic!("Can't load history for {}: {}", symbol, e),
                }
            })
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::domain::{Day, History};
    use implementation::*;

    struct MockHistoricalDataService {}
    impl HistoricalDataService for MockHistoricalDataService {
        fn fetch(
            &self,
            symbol: &str,
            start_date: NaiveDate,
            end_date: NaiveDate,
        ) -> Result<History, reqwest::Error> {
            Ok(History {
                day: vec![
                    Day {
                        date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
                        open: 1.0,
                        high: 1.0,
                        low: 1.0,
                        close: 10.0,
                        volume: 1,
                    },
                    Day {
                        date: NaiveDate::from_ymd_opt(2024, 4, 2).unwrap(),
                        open: 2.0,
                        high: 2.0,
                        low: 2.0,
                        close: 10.0,
                        volume: 2,
                    },
                    Day {
                        date: NaiveDate::from_ymd_opt(2024, 4, 3).unwrap(),
                        open: 3.0,
                        high: 3.0,
                        low: 3.0,
                        close: 20.0,
                        volume: 3,
                    },
                ],
            })
        }
    }

    #[test]
    fn test_load_history() {
        let symbols = vec!["AAPL".to_string()];
        let historical_data_service = Arc::new(MockHistoricalDataService {});
        // let trading_service = new(
        //     "Test".to_string(),
        //     &symbols,
        //     Arc::new(MockMarketDataService {}),
        //     historical_data_service.clone(),
        // );
        let data = load_history(&symbols, historical_data_service);
        let aapl = data.get(&"AAPL".to_string()).unwrap();
        assert_eq!(aapl.mean, 13.333333333333334);
        assert_eq!(aapl.std_dev, 4.714045207910316);
    }
}
