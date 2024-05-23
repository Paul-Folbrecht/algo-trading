#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashSet;

use config::AppConfig;
use trading::TradingService;

mod config;
mod historical_data;
mod market_data;
mod serde;
mod strategy;
mod trading;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);
    let args: Vec<String> = std::env::args().collect();
    let access_token = &args[1];
    let market_data_service = market_data::new(access_token.to_string());
    let mut symbols: HashSet<String> = HashSet::new();

    config.strategies.iter().for_each(|strategy| {
        symbols.extend(strategy.symbols.clone());
        let mut trading_service = trading::new(
            strategy.name.clone(),
            strategy.symbols.clone(),
            market_data_service.clone(),
        );
        trading_service.run().unwrap();
    });
    // @todo symbls should be collected in main() and passed to market_data_service
    let handle = market_data_service.init(symbols.into_iter().collect());
    handle.unwrap().join().unwrap();
}
