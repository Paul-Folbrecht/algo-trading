use std::sync::Arc;

use chrono::NaiveDate;
use domain::domain::Day;
use services::historical_data::HistoricalDataService;
use std::collections::HashMap;

pub fn new(
    access_token: String,
    symbols: Vec<String>,
    backtest_range: i64,
    hist_data_range: i64,
    end: NaiveDate,
    underlying: Arc<impl HistoricalDataService + 'static + Send + Sync>,
) -> Arc<impl HistoricalDataService> {
    let start = end - chrono::Duration::days(backtest_range + hist_data_range);
    let history = symbols
        .iter()
        .map(|symbol| {
            let data = underlying
                .fetch(symbol, start, end)
                .expect("Failed to fetch historical data");
            (symbol.clone(), data)
        })
        .collect::<HashMap<String, Vec<Day>>>();
    Arc::new(implementation::BacktestHistoricalData { start, history })
}

mod implementation {
    use super::*;

    pub struct BacktestHistoricalData {
        pub start: NaiveDate,
        pub history: HashMap<String, Vec<Day>>,
    }

    impl HistoricalDataService for BacktestHistoricalData {
        fn fetch(
            &self,
            symbol: &str,
            start: NaiveDate,
            end: NaiveDate,
        ) -> reqwest::Result<Vec<Day>> {
            let symbol_history = self
                .history
                .get(symbol)
                .expect(format!("No data for {}", symbol).as_ref());
            let start_index = date_to_index(start, self.start);
            let end_index = date_to_index(end, self.start);

            println!(
                "BacktestHistoricalData.fetch: from {} to {}; indices {} - {}; len {}",
                start,
                end,
                start_index,
                end_index,
                symbol_history.len()
            );

            assert!(start_index >= 0);
            assert!((end_index as usize) < symbol_history.len());
            Ok(symbol_history[start_index as usize..end_index as usize].to_vec())
        }
    }

    fn date_to_index(date: NaiveDate, start: NaiveDate) -> i64 {
        assert!(date >= start);
        (date - start).num_days()
    }
}

#[cfg(test)]
#[path = "./tests/backtest_historical_data_test.rs"]
mod backtest_historical_data_test;
