use super::*;
use chrono::NaiveDate;

#[test]
fn test_fetch() {
    let access_token = std::env::var("TRADIER_ACCESS_TOKEN").unwrap();
    let symbol = "SPY";
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
    let service = new(access_token);
    match service.fetch(symbol, start, end) {
        Ok(history) => {
            println!("History: {:?}", history);
            assert!(history.len() > 0);
        }
        Err(e) => {
            println!("Error: {}", e);
            assert!(false);
        }
    }
}
