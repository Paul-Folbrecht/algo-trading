use std::sync::Arc;

use chrono::NaiveDate;
use domain::domain::Day;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;

pub trait HistoricalDataService {
    fn fetch(&self, symbol: &str, start: NaiveDate, end: NaiveDate) -> reqwest::Result<Vec<Day>>;
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
    struct HistoryResponse {
        pub history: History,
    }

    #[derive(Deserialize, Debug)]
    struct History {
        pub day: Vec<Day>,
    }

    impl HistoricalDataService for HistoricalData {
        fn fetch(
            &self,
            symbol: &str,
            start: NaiveDate,
            end: NaiveDate,
        ) -> reqwest::Result<Vec<Day>> {
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
                    Ok(history) => Ok(history.history.day),
                    Err(e) => {
                        eprintln!("Failed to deserialize: {}", e);
                        Err(e)
                    }
                },
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/historical_data_test.rs"]
mod historical_data_test;
