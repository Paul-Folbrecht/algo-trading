#![allow(dead_code)]
#![allow(unused_variables)]

use app_config::app_config::AppConfig;
use backtest_service::BacktestService;
use chrono::Local;
use core::util::time;
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
        end,
        config.backtest_range,
        config.hist_data_range,
        historical_data,
    );

    let backtest_market_data_manager = backtest_market_data_manager::new(
        config.access_token.clone(),
        symbols.clone(),
        config.backtest_range,
        end,
        backtest_historical_data.clone(),
    );

    let backtest_service = backtest_service::new(
        end,
        config.backtest_range,
        backtest_historical_data.clone(),
        backtest_market_data_manager,
        backtest_orders::new(),
        config.strategies.clone(),
    );

    time("backtest_service.run()", || match backtest_service.run() {
        Ok(_) => println!("Backtest completed successfully"),
        Err(e) => eprintln!("Backtest failed: {}", e),
    })
}
