use super::*;
use chrono::NaiveDate;
use mock_historical_data_service::MockHistoricalDataService;

#[test]
fn test_windowing() {
    let symbols = vec!["SPY".to_string()];
    let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    let historical_data_service = Arc::new(MockHistoricalDataService { end });
    let backtest_range = 20;
    let hist_data_range = 4;
    let service = Arc::new(new(
        "".to_string(),
        symbols,
        backtest_range,
        hist_data_range,
        end,
        historical_data_service,
    ));

    // First day of backtest range
    let start =
        NaiveDate::from_ymd_opt(2024, 6, (30 - backtest_range - hist_data_range) as u32).unwrap();
    let end = start + chrono::Duration::days(4);
    let data = service.fetch("SPY", start, end).unwrap();
    assert_eq!(data.len(), hist_data_range as usize);
    assert_eq!(
        data.into_iter().map(|d| d.date).collect::<Vec<_>>(),
        vec![
            NaiveDate::from_ymd_opt(2024, 6, 6).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 7).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 8).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 9).unwrap()
        ]
    );

    // Middle
    let start = NaiveDate::from_ymd_opt(2024, 6, 16).unwrap();
    let end = start + chrono::Duration::days(4);
    let data = service.fetch("SPY", start, end).unwrap();
    assert_eq!(data.len(), hist_data_range as usize);
    assert_eq!(
        data.into_iter().map(|d| d.date).collect::<Vec<_>>(),
        vec![
            NaiveDate::from_ymd_opt(2024, 6, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 17).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 18).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 19).unwrap()
        ]
    );

    // Second to last

    // Last
    let start = NaiveDate::from_ymd_opt(2024, 6, 26).unwrap();
    let end = start + chrono::Duration::days(4);
    let data = service.fetch("SPY", start, end).unwrap();
    assert_eq!(data.len(), hist_data_range as usize);
    assert_eq!(
        data.into_iter().map(|d| d.date).collect::<Vec<_>>(),
        vec![
            NaiveDate::from_ymd_opt(2024, 6, 26).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 27).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 28).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 29).unwrap()
        ]
    );
}
