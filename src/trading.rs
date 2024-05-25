use std::sync::Arc;

use crate::market_data::*;
use crate::strategy::*;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new(
    strategy_name: String,
    symbols: Vec<String>,
    market_data_service: Arc<impl MarketDataService>,
) -> impl TradingService {
    let strategy = crate::strategy::Strategy::new(strategy_name, symbols);
    Trading {
        strategy,
        market_data_service,
        thread_handle: None,
    }
}

pub struct Trading<M: MarketDataService> {
    strategy: Strategy,
    market_data_service: Arc<M>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

mod implementation {
    use super::*;

    impl<M: MarketDataService> TradingService for Trading<M> {
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
