#![allow(dead_code)]
#![allow(unused_variables)]

use app_config::app_config::AppConfig;
use backtest_orders::BacktestOrderService;
use backtest_service::BacktestService;
use chrono::Local;
use core::util::time;
use itertools::Itertools;
use log::*;
use log4rs;
use services::historical_data;
use std::env;

mod backtest_historical_data;
mod backtest_market_data_manager;
mod backtest_orders;
mod backtest_service;

fn main() {
    let access_token = env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN not found");

    log4rs::init_file("config/backtest-log4rs.yaml", Default::default()).unwrap();
    let config = AppConfig::new().expect("Could not load config");
    info!("Config:\n{:?}", config);

    let end = Local::now().naive_local().date();
    let symbols = config.all_symbols();

    let historical_data = historical_data::new(
        access_token.clone(),
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
        access_token.clone(),
        symbols.clone(),
        config.backtest_range,
        end,
        backtest_historical_data.clone(),
    );

    let orders = backtest_orders::new();
    let backtest_service = backtest_service::new(
        end,
        config.backtest_range,
        backtest_historical_data.clone(),
        backtest_market_data_manager,
        orders.clone(),
        config.strategies.clone(),
    );

    time("backtest_service.run()", || match backtest_service.run() {
        Ok(_) => {
            let pnl = orders.realized_pnl();
            info!(
                "\nBacktest completed successfully\n\nOpen positions:\n{:?}\n\nRealized P&L:\n{:?}\n\nTotal P&L: {}\n",
                orders.open_positions().iter().format("\n"),
                pnl.iter().format("\n"),
                pnl.iter().map(|pnl| pnl.pnl).sum::<f64>());
        }
        Err(e) => info!("Backtest failed: {}", e),
    })
}
