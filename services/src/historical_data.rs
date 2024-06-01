use std::sync::Arc;

use chrono::NaiveDate;
use domain::domain::History;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;

pub trait HistoricalDataService {
    fn fetch(&self, symbol: &str, start: NaiveDate, end: NaiveDate) -> reqwest::Result<History>;
}

pub fn new(access_token: String) -> Arc<impl HistoricalDataService> {
    Arc::new(implementation::HistoricalData { access_token })
}

mod implementation {
    use super::*;

    pub struct HistoricalData {
        pub access_token: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct HistoryResponse {
        pub history: History,
    }

    impl HistoricalDataService for HistoricalData {
        fn fetch(
            &self,
            symbol: &str,
            start: NaiveDate,
            end: NaiveDate,
        ) -> reqwest::Result<History> {
            let base = "https://api.tradier.com/v1/markets/history";
            let params = format!(
                "symbol={}&interval=daily&start={}&end={}&session_filter=all",
                symbol, start, end
            );
            let url = format!("{}?{}", base, params);
            match reqwest::blocking::Client::new()
                .get(url.as_str())
                .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
                .header(ACCEPT, "application/json")
                .header(CONTENT_LENGTH, "0")
                .send()
            {
                Ok(response) => match response.json::<implementation::HistoryResponse>() {
                    Ok(history) => Ok(history.history),
                    Err(e) => {
                        println!("Failed to deserialize: {}", e);
                        Err(e)
                    }
                },
                Err(e) => {
                    println!("Request failed: {}", e);
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/historical_data_test.rs"]
mod historical_data_test;
