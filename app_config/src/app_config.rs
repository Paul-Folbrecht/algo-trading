use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::{collections::HashMap, env};

#[derive(Debug)]
pub struct AppConfig {
    pub access_token: String,
    pub sandbox_token: String,
    pub account_id: String,
    pub sandbox: bool,
    pub mongo_url: String,
    pub strategies: Vec<Strategy>,
    pub hist_data_range: i64,
    pub backtest_range: i64,
}

impl AppConfig {
    pub fn all_symbols(&self) -> Vec<String> {
        self.strategies
            .iter()
            .flat_map(|s| s.symbols.clone())
            .collect()
    }
}

impl From<ConfigHolder> for AppConfig {
    fn from(holder: ConfigHolder) -> Self {
        AppConfig {
            access_token: holder.access_token,
            sandbox_token: holder.sandbox_token,
            account_id: holder.account_id,
            sandbox: holder.sandbox,
            mongo_url: holder.mongo_url,
            strategies: holder.strategies.into_iter().map(|s| s.into()).collect(),
            hist_data_range: holder.hist_data_range,
            backtest_range: holder.backtest_range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub symbols: Vec<String>,
    pub capital: HashMap<String, i64>,
}

impl From<StrategyHolder> for Strategy {
    fn from(holder: StrategyHolder) -> Self {
        let capital = holder
            .symbols
            .iter()
            .zip(holder.capital.iter())
            .map(|(s, c)| (s.clone(), *c))
            .collect();

        Strategy {
            name: holder.name,
            symbols: holder.symbols,
            capital,
        }
    }
}

#[derive(Deserialize)]
struct ConfigHolder {
    pub access_token: String,
    pub sandbox_token: String,
    pub account_id: String,
    pub sandbox: bool,
    pub mongo_url: String,
    pub strategies: Vec<StrategyHolder>,
    pub hist_data_range: i64,
    pub backtest_range: i64,
}

#[derive(Deserialize)]
struct StrategyHolder {
    pub name: String,
    pub symbols: Vec<String>,
    pub capital: Vec<i64>,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let holder: ConfigHolder = Config::builder()
            .add_source(File::with_name("app_config/default"))
            .add_source(File::with_name(&format!("/app_config/{}", run_mode)).required(false))
            .add_source(File::with_name("app_config/local").required(false))
            .build()?
            .try_deserialize::<ConfigHolder>()?;
        Ok(holder.into())
    }
}
