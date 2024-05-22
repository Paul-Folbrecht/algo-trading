use std::sync::Arc;

use crate::market_data::*;
use crate::strategy::*;

pub trait TradingService {
    fn run(&mut self) -> Result<(), String>;
}

pub fn new(
    strategy_name: String,
    symbols: Vec<String>,
    market_data_service: Arc<dyn MarketDataService>,
    //market_data_service: &impl MarketDataService,
) -> impl TradingService {
    let strategy = crate::strategy::Strategy::new(strategy_name, symbols);
    Trading {
        strategy,
        market_data_service,
    }
}

//pub struct Trading <M: MarketDataService> {
pub struct Trading {
    strategy: Strategy,
    market_data_service: Arc<dyn MarketDataService>,
    //market_data_service: &M,
}

mod implementation {
    use super::*;

    //    impl<M: MarketDataService> TradingService for Trading<M> {
    impl TradingService for Trading {
        fn run(&mut self) -> Result<(), String> {
            println!("Running TradingService with strategy: {:?}", self.strategy);
            self.market_data_service.subscribe()?;
            Ok(())
        }
    }
}
