use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Strategy {
    pub name: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub strategy: Strategy,
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
