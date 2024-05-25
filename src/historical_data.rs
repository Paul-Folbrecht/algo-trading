use std::sync::Arc;

use crate::serde::tradier_date_format;
use chrono::NaiveDate;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;

pub trait HistoricalDataService {
    fn fetch(&self, symbol: &str, start: NaiveDate, end: NaiveDate) -> reqwest::Result<History>;
}

pub fn new(access_token: String) -> Arc<impl HistoricalDataService> {
    Arc::new(implementation::HistoricalData { access_token })
}

#[derive(Deserialize, Debug)]
pub struct History {
    day: Vec<Day>,
}

#[derive(Deserialize, Debug)]
pub struct Day {
    #[serde(with = "tradier_date_format")]
    date: NaiveDate,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
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
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_fetch() {
        let access_token = std::env::var("TRADIER_ACCESS_TOKEN").unwrap();
        let symbol = "AAPL";
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
        let service = new(access_token);
        match service.fetch(symbol, start, end) {
            Ok(history) => {
                println!("History: {:?}", history);
                assert!(history.day.len() > 0);
            }
            Err(e) => {
                println!("Error: {}", e);
                assert!(false);
            }
        }
    }
}
