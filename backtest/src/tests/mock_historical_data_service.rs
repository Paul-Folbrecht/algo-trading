use std::{collections::HashMap, sync::Arc};

use chrono::{Datelike, NaiveDate};
use domain::domain::Day;
use services::historical_data::HistoricalDataService;

pub struct MockHistoricalDataService {
    pub end: NaiveDate,
}

impl HistoricalDataService for MockHistoricalDataService {
    fn fetch(&self, end: NaiveDate) -> Arc<HashMap<String, Vec<Day>>> {
        let backtest_range = 20;
        let hist_data_range = 4;
        let start = self.end - chrono::Duration::days(backtest_range + hist_data_range);
        let mut days = Vec::with_capacity((backtest_range + hist_data_range) as usize);
        for i in 0..=backtest_range + hist_data_range {
            let date = start + chrono::Duration::days(i);
            days.push(Day {
                symbol: Some("SPY".to_string()),
                date: date,
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: date.day() as f64,
                volume: 10000,
            });
        }
        let mut map = HashMap::new();
        map.insert("SPY".to_string(), days);
        Arc::new(map)
    }
}
