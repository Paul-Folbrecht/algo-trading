use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::{collections::HashMap, env};

#[derive(Debug, Deserialize)]
pub struct Strategy {
    pub name: String,
    pub symbols: Vec<String>,
    pub capital: HashMap<String, i64>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub access_token: String,
    pub sandbox_token: String,
    pub account_id: String,
    pub sandbox: bool,
    pub strategies: Vec<Strategy>,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("/config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .build()?
            .try_deserialize()
    }
}
