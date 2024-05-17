use config::AppConfig;
use market_data::MarketDataService;

mod config;
mod market_data;

fn main() {
    let config = AppConfig::new();
    let args: Vec<String> = std::env::args().collect();
    let access_token = &args[1];
    let mut market_data_service = market_data::new("access_token".to_string());
    market_data_service.init(vec!["AAPL".to_string(), "MSFT".to_string()]);
}
