#![allow(dead_code)]
#![allow(unused_variables)]

use app_config::app_config::AppConfig;

fn main() {
    println!("Backtesting!");
    let config = AppConfig::new().expect("Could not load config");
    println!("Config:\n{:?}", config);
}
