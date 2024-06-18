#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Arc},
};

use config::AppConfig;
use services::market_data::MarketDataService;
use services::trading::TradingService;
use services::{historical_data, market_data, orders, trading};

mod config;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);
    // let args: Vec<String> = std::env::args().collect();
    // let access_token = &args[1];
    let access_token = config.access_token;
    let market_data_service = market_data::new(access_token.clone());
    let historical_data_service = historical_data::new(access_token.clone());
    let order_service = orders::new(
        access_token.clone(),
        config.account_id.clone(),
        config.sandbox,
    );
    let shutdown = Arc::new(AtomicBool::new(false));
    let mut symbols: HashSet<String> = HashSet::new();

    config.strategies.iter().for_each(|strategy| {
        symbols.extend(strategy.symbols.clone());
        let mut trading_service = trading::new(
            strategy.name.clone(),
            strategy.symbols.as_ref(),
            market_data_service.clone(),
            historical_data_service.clone(),
        );
        match trading_service.run() {
            Ok(_) => (),
            Err(e) => eprintln!("Error starting TradingService {}: {}", strategy.name, e),
        }
    });

    let handle = market_data_service.init(shutdown, symbols.into_iter().collect());
    match handle.unwrap().join() {
        Ok(_) => println!("MarketDataService thread exited successfully"),
        Err(e) => eprintln!("Error joining MarketDataService thread: {:?}", e),
    }
}
