use crate::serde::tradier_date_format;
use chrono::NaiveDate;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HistoryResponse {
    history: History,
}

#[derive(Deserialize, Debug)]
pub struct History {
    day: Vec<Day>,
}

#[derive(Deserialize, Debug)]
pub struct Day {
    // #[serde(with = "tradier_date_format")]
    // date: NaiveDate,
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
}

pub fn fetch(
    access_token: &str,
    symbol: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> reqwest::Result<History> {
    // ?symbol=AAPL&interval=daily&start=2019-05-04&end=2019-05-04&session_filter=all" \
    let base = "https://api.tradier.com/v1/markets/history";
    let params = format!(
        "symbol={}&interval=daily&start={}&end={}&session_filter=all",
        symbol, start, end
    );
    let url = format!("{}?{}", base, params);
    println!(
        "response:\n{}",
        reqwest::blocking::Client::new()
            .get(url.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .header(CONTENT_LENGTH, "0")
            .send()?
            .text()?
    );
    match reqwest::blocking::Client::new()
        .get(url.as_str())
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(ACCEPT, "application/json")
        .header(CONTENT_LENGTH, "0")
        .send()
    {
        Ok(response) => match response.json::<HistoryResponse>() {
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
        match fetch(&access_token, symbol, start, end) {
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
