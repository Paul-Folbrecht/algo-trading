#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Arc},
};

use config::AppConfig;
use services::persistence::PersistenceService;
use services::trading::TradingService;
use services::{historical_data, market_data, orders, trading};
use services::{market_data::MarketDataService, persistence};

mod config;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);

    let access_token = config.access_token;
    let market_data = market_data::new(access_token.clone());
    let historical_data = historical_data::new(access_token.clone());
    let persistence = persistence::new();
    let orders = orders::new(
        access_token.clone(),
        config.account_id.clone(),
        config.sandbox,
        persistence.clone(),
    )
    .expect("Failed to create OrdersService");
    let shutdown = Arc::new(AtomicBool::new(false));
    let mut symbols: HashSet<String> = HashSet::new();

    config.strategies.into_iter().for_each(|strategy| {
        symbols.extend(strategy.symbols.clone());
        let mut trading_service = trading::new(
            strategy.name.clone(),
            strategy.symbols.clone(),
            market_data.clone(),
            historical_data.clone(),
            orders.clone(),
        );
        match trading_service.run() {
            Ok(_) => (),
            Err(e) => eprintln!("Error starting TradingService {}: {}", strategy.name, e),
        }
    });

    let handle1 = persistence.init(shutdown.clone());
    let handle2 = market_data.init(shutdown.clone(), symbols.into_iter().collect());
    let handles = vec![handle1, handle2];
    handles
        .into_iter()
        .for_each(|handle| match handle.unwrap().join() {
            Ok(_) => println!("Thread exited successfully"),
            Err(e) => eprintln!("Error joining thread: {:?}", e),
        });
    println!("All threads exited successfully");
}
