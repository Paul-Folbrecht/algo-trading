#![allow(dead_code)]
#![allow(unused_variables)]

use config::AppConfig;
use market_data::MarketDataService;
use trading::TradingService;

mod config;
mod market_data;
mod strategy;
mod tradier_date_format;
mod trading;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);
    let args: Vec<String> = std::env::args().collect();
    let access_token = &args[1];
    let market_data_service = market_data::new(access_token.to_string());
    // let mut market_data_service = market_data::new2(access_token.to_string());

    // Each TradingService will subscribe to passed MarketDataService
    // When MarketDataService is started it'll start sending to TradingServices
    config.strategies.iter().for_each(|strategy| {
        let mut trading_service = trading::new(
            strategy.name.clone(),
            strategy.symbols.clone(),
            market_data_service.clone(),
        );
        trading_service.run().unwrap();
    });
    let handle = market_data_service.init(vec!["AAPL".to_string(), "MSFT".to_string()]);
    handle.unwrap().join().unwrap();
}
