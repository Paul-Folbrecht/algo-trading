#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashSet,
    env,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use app_config::app_config::AppConfig;
use chrono::{Local, NaiveDate};
use log::*;
use log4rs;
use services::persistence::PersistenceService;
use services::trading::TradingService;
use services::{historical_data, market_data, orders, trading};
use services::{market_data::MarketDataService, persistence};

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default())
        .expect("Failed to load log4rs config");
    let config = AppConfig::new().expect("Failed to parse config");
    info!("Config:\n{:?}", config);

    let mut today = Local::now().naive_local().date();
    let (mut shutdown, mut handle) = init_for_new_day(today, config.clone());

    loop {
        thread::sleep(Duration::from_secs(300));
        let now = Local::now().naive_local().date();

        if now > today {
            shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

            handle
                .join()
                .expect("Failed to join MarketDataService thread");
            info!("All threads exited successfully");

            today = now;
            info!("Trading day ended - resetting for {}", today);
            (shutdown, handle) = init_for_new_day(today, config.clone());
        }
    }
}

fn init_for_new_day(today: NaiveDate, config: AppConfig) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let access_token = env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN not found");
    let sandbox_token = env::var("SANDBOX_TOKEN").expect("SANDBOX_TOKEN not found");
    let account_id = env::var("ACCOUNT_ID").expect("ACCOUNT_ID not found");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL not found");
    let market_data = market_data::new(access_token.clone());
    let symbols = config.all_symbols();
    let shutdown = Arc::new(AtomicBool::new(false));
    let persistence = persistence::new(mongo_url.clone());

    persistence
        .init(shutdown.clone())
        .expect("Failed to initialize persistence");

    let historical_data = historical_data::new(
        access_token.clone(),
        symbols.clone(),
        config.hist_data_range,
        today,
    );

    let orders = if config.sandbox {
        orders::new(
            sandbox_token.clone(),
            account_id.clone(),
            "sandbox.tradier.com".into(),
            persistence.clone(),
        )
        .expect("Failed to create OrdersService")
    } else {
        orders::new(
            access_token.clone(),
            account_id.clone(),
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
            Err(e) => info!("Error starting TradingService {}: {}", strategy.name, e),
        }
    });

    let handle = market_data
        .init(shutdown.clone(), symbols.into_iter().collect())
        .expect("Failed to start MarketDataService");

    (shutdown, handle)
}
