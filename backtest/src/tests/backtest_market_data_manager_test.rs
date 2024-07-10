use super::*;
use chrono::Datelike;
use mock_historical_data_service::MockHistoricalDataService;

#[test]
fn test_backtest_market_data_manager() {
    let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    let historical_data_service = Arc::new(MockHistoricalDataService { end });
    let backtest_range = 20;
    let symbols = vec!["AAPL".to_string(), "MSFT".to_string()];
    let service = Arc::new(new(
        "".to_string(),
        symbols,
        backtest_range,
        end,
        historical_data_service,
    ));

    let start_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
    let mut date = start_date;
    while date <= end_date {
        println!("{:?}", date);
        date += Duration::days(1);
        let market_data = service.service_for_date(date);
        let receiver = market_data.subscribe().expect("Failed to subscribe");
        let quote = receiver.recv().expect("Failed to receive quote");
        assert_eq!(quote.symbol, "SPY");
        assert_eq!(quote.bid, date.day() as f64);
        assert_eq!(quote.ask, date.day() as f64);
    }
}
