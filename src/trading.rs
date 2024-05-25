use std::sync::Arc;

use crate::historical_data::*;
use crate::market_data::*;
use crate::strategy::*;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new(
    strategy_name: String,
    symbols: Vec<String>,
    market_data_service: Arc<impl MarketDataService>,
    historical_data_service: Arc<impl HistoricalDataService>,
) -> impl TradingService {
    let strategy = crate::strategy::Strategy::new(strategy_name, symbols);
    Trading {
        strategy,
        market_data_service,
        historical_data_service,
        thread_handle: None,
    }
}

pub struct Trading<M: MarketDataService, H: HistoricalDataService> {
    strategy: Strategy,
    market_data_service: Arc<M>,
    historical_data_service: Arc<H>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

mod implementation {
    use super::*;

    impl<M: MarketDataService, H: HistoricalDataService> TradingService for Trading<M, H> {
        fn run(&mut self) -> Result<(), String> {
            println!("Running TradingService with strategy: {:?}", self.strategy);
            match self.market_data_service.subscribe() {
                Ok(rx) => {
                    println!("Subscribed to MarketDataService");
                    self.thread_handle = Some(std::thread::spawn(move || loop {
                        match rx.recv() {
                            Ok(quote) => {
                                println!("TradingService received quote:\n{:?}", quote);
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
}
