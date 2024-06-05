use super::*;
use chrono::NaiveDate;
use domain::domain::{Day, History};
use implementation::*;

struct MockHistoricalDataService {}
impl HistoricalDataService for MockHistoricalDataService {
    fn fetch(&self, _: &str, _: NaiveDate, _: NaiveDate) -> Result<History, reqwest::Error> {
        Ok(History {
            day: vec![
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
                    open: 1.0,
                    high: 1.0,
                    low: 1.0,
                    close: 10.0,
                    volume: 1,
                },
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 2).unwrap(),
                    open: 2.0,
                    high: 2.0,
                    low: 2.0,
                    close: 10.0,
                    volume: 2,
                },
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 3).unwrap(),
                    open: 3.0,
                    high: 3.0,
                    low: 3.0,
                    close: 20.0,
                    volume: 3,
                },
            ],
        })
    }
}

#[test]
fn test_load_history() {
    let symbols = vec!["AAPL".to_string()];
    let historical_data_service = Arc::new(MockHistoricalDataService {});
    let data = load_history(&symbols, historical_data_service);
    let aapl = data.get(&"AAPL".to_string()).unwrap();
    assert_eq!(aapl.mean, 13.333333333333334);
    assert_eq!(aapl.std_dev, 4.714045207910316);
}
