use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
use domain::domain::Day;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;

// hist service fetches on init, all symbols, map symbol to vec of days
// fetch() takes end date; range is from config
pub trait HistoricalDataService {
    fn fetch(&self, end: NaiveDate) -> Arc<HashMap<String, Vec<Day>>>;
}

pub fn new(
    access_token: String,
    symbols: Vec<String>,
    range: i64,
    end: NaiveDate,
) -> Arc<impl HistoricalDataService> {
    let history = fetch(access_token.as_str(), symbols, range, end);
    Arc::new(implementation::HistoricalData {
        access_token,
        history: Arc::new(history),
    })
}

pub fn fetch(
    access_token: &str,
    symbols: Vec<String>,
    range: i64,
    end: NaiveDate,
) -> HashMap<String, Vec<Day>> {
    let start = end - chrono::Duration::days(range);
    symbols
        .iter()
        .map(|symbol| {
            let data = implementation::fetch_one(access_token, symbol, start, end)
                .expect("Failed to fetch historical data");
            (symbol.clone(), data)
        })
        .collect::<HashMap<String, Vec<Day>>>()
}

mod implementation {
    use super::*;

    pub struct HistoricalData {
        pub access_token: String,
        pub history: Arc<HashMap<String, Vec<Day>>>,
    }

    #[derive(Deserialize, Debug)]
    struct HistoryResponse {
        pub history: History,
    }

    #[derive(Deserialize, Debug)]
    struct History {
        pub day: Vec<Day>,
    }

    pub fn fetch_one(
        access_token: &str,
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
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .header(CONTENT_LENGTH, "0")
            .send()
        {
            Ok(response) => match response.json::<implementation::HistoryResponse>() {
                Ok(history) => {
                    let with_symbols = history
                        .history
                        .day
                        .iter()
                        .map(|day| Day {
                            symbol: Some(symbol.to_string()),
                            ..day.clone()
                        })
                        .collect();
                    Ok(with_symbols)
                }
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

    impl HistoricalDataService for HistoricalData {
        fn fetch(&self, _: NaiveDate) -> Arc<HashMap<String, Vec<Day>>> {
            // We can ignore 'end' for the live-trading case because it's always the current date
            self.history.clone()
        }
    }
}

#[cfg(test)]
#[path = "./tests/historical_data_test.rs"]
mod historical_data_test;
