use std::sync::Arc;

use crate::historical_data::*;
use crate::market_data::*;
use domain::domain::*;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new<'market_data>(
    strategy_name: String,
    symbols: &'market_data Vec<String>,
    market_data_service: Arc<impl MarketDataService + 'market_data + Send + Sync>,
    historical_data_service: Arc<impl HistoricalDataService + 'market_data + Send + Sync>,
) -> impl TradingService + 'market_data {
    implementation::Trading {
        strategy_name,
        symbols,
        market_data_service,
        historical_data_service,
        thread_handle: None,
    }
}

mod implementation {
    use super::*;
    use chrono::Local;
    use domain::domain::SymbolData;
    use std::collections::HashMap;

    pub struct Trading<
        'market_data,
        M: MarketDataService + Send + Sync,
        H: HistoricalDataService + Send + Sync,
    > {
        pub strategy_name: String,
        pub symbols: &'market_data Vec<String>,
        pub market_data_service: Arc<M>,
        pub historical_data_service: Arc<H>,
        pub thread_handle: Option<std::thread::JoinHandle<()>>,
    }

    impl<
            'market_data,
            M: MarketDataService + Send + Sync,
            H: HistoricalDataService + Send + Sync,
        > TradingService for Trading<'market_data, M, H>
    {
        fn run(&mut self) -> Result<(), String> {
            println!(
                "Running TradingService with strategy: {:?}",
                self.strategy_name
            );
            let symbol_data = load_history(self.symbols, self.historical_data_service.clone());

            match self.market_data_service.subscribe() {
                Ok(rx) => {
                    println!("TradingService subscribed to MarketDataService");
                    let strategy = Strategy::new(&self.strategy_name, self.symbols.clone());
                    self.thread_handle = Some(std::thread::spawn(move || loop {
                        match rx.recv() {
                            Ok(quote) => {
                                println!("TradingService received quote:\n{:?}", quote);
                                if let Some(symbol_data) = symbol_data.get(&quote.symbol) {
                                    strategy.handle(&quote, symbol_data);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error on receive!: {}", e);
                            }
                        }
                    }));
                }

                Err(e) => return Err(format!("Failed to subscribe to MarketDataService: {}", e)),
            }
            Ok(())
        }
    }

    fn load_history<'market_data>(
        symbols: &'market_data Vec<String>,
        historical_data_service: Arc<impl HistoricalDataService + 'market_data>,
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
#[path = "./../tests/trading_test.rs"]
mod trading_test;
