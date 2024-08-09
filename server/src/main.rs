#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use app_config::app_config::AppConfig;
use chrono::Local;
use services::persistence::PersistenceService;
use services::trading::TradingService;
use services::{historical_data, market_data, orders, trading};
use services::{market_data::MarketDataService, persistence};

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);

    loop {
        let (shutdown, handle) = init_for_today(config.clone());

        // sleep for 5 minutes, check if time > midnight
        // if time > midnight:
        // - set shutdown to true
        // - join all threads - market_data, trading_services
        // - call init_for_today again, store new shutdown
        thread::sleep(Duration::from_secs(60));
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        println!("Trading day ended - resetting");

        handle
            .join()
            .expect("Failed to join MarketDataService thread");
        println!("All threads exited successfully");
        break;
    }
}

fn init_for_today(config: AppConfig) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let today = Local::now().naive_local().date();
    let market_data = market_data::new(config.access_token.clone());
    let symbols = config.all_symbols();
    let shutdown = Arc::new(AtomicBool::new(false));
    let persistence = persistence::new(config.mongo_url.clone());

    persistence
        .init(shutdown.clone())
        .expect("Failed to initialize persistence");

    let historical_data = historical_data::new(
        config.access_token.clone(),
        symbols.clone(),
        config.hist_data_range,
        today,
    );

    let orders = if config.sandbox {
        orders::new(
            config.sandbox_token.clone(),
            config.account_id.clone(),
            "sandbox.tradier.com".into(),
            persistence.clone(),
        )
        .expect("Failed to create OrdersService")
    } else {
        orders::new(
            config.access_token.clone(),
            config.account_id.clone(),
            "api.tradier.com".into(),
            persistence.clone(),
        )
        .expect("Failed to create OrdersService")
    };

    let mut symbols: HashSet<String> = HashSet::new();
    let date = Local::now().naive_local().date();

    config.strategies.into_iter().for_each(|strategy| {
        symbols.extend(strategy.symbols.clone());
        let mut trading_service = trading::new(
            date,
            strategy.name.clone(),
            strategy.symbols.clone(),
            strategy.capital.clone(),
            market_data.clone(),
            historical_data.clone(),
            orders.clone(),
            shutdown.clone(),
        );
        match trading_service.run() {
            Ok(_) => (),
            Err(e) => println!("Error starting TradingService {}: {}", strategy.name, e),
        }
    });

    let handle = market_data
        .init(shutdown.clone(), symbols.into_iter().collect())
        .expect("Failed to start MarketDataService");

    (shutdown, handle)
}
