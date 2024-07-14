#![allow(dead_code)]
#![allow(unused_variables)]

use app_config::app_config::AppConfig;
use chrono::Local;
use services::historical_data;

mod backtest_historical_data;
mod backtest_market_data_manager;
mod backtest_orders;
mod backtest_service;

fn main() {
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);

    let end = Local::now().naive_local().date();
    let symbols = config.all_symbols();
    let historical_data = historical_data::new(
        config.access_token.clone(),
        symbols.clone(),
        config.backtest_range + config.hist_data_range,
        end,
    );
    let backtest_historical_data = backtest_historical_data::new(
        config.backtest_range,
        config.hist_data_range,
        historical_data,
    );

    let backtest_market_data_manager = backtest_market_data_manager::new(
        config.access_token.clone(),
        symbols.clone(),
        config.backtest_range,
        end,
        backtest_historical_data,
    );
}
