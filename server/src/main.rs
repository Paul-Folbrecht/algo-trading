#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Arc},
};

use config::AppConfig;
use services::historical_data;
use services::market_data;
use services::market_data::MarketDataService;
use services::trading;
use services::trading::TradingService;

mod config;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);
    let args: Vec<String> = std::env::args().collect();
    let access_token = &args[1];
    let market_data_service = market_data::new(access_token.to_string());
    let histiorical_data_service = historical_data::new(access_token.to_string());
    let shutdown = Arc::new(AtomicBool::new(false));
    let mut symbols: HashSet<String> = HashSet::new();

    config.strategies.iter().for_each(|strategy| {
        symbols.extend(strategy.symbols.clone());
        let mut trading_service = trading::new(
            strategy.name.clone(),
            strategy.symbols.clone(),
            market_data_service.clone(),
            histiorical_data_service.clone(),
        );
        trading_service.run().unwrap();
    });
    let handle = market_data_service.init(shutdown, symbols.into_iter().collect());
    handle.unwrap().join().unwrap();
}