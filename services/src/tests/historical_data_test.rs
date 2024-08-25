use super::*;
use chrono::NaiveDate;

#[test]
fn test_fetch() {
    let access_token = std::env::var("TRADIER_ACCESS_TOKEN").unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
    let service = new(access_token, vec!["SPY".to_string()], 20, end);
    match service.fetch(end).get("SPY") {
        Some(history) => {
            println!("History: {:?}", history);
            assert!(history.len() > 0);
        }
        None => {
            assert!(false);
        }
    }
}
