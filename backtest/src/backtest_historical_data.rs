use chrono::NaiveDate;
use domain::domain::Day;
use services::historical_data::HistoricalDataService;
use std::collections::HashMap;
use std::sync::Arc;

pub trait BacktestHistoricalDataManager: HistoricalDataService {
    fn all(&self) -> Arc<HashMap<String, Vec<Day>>>;
}

pub fn new(
    end: NaiveDate,
    range: i64,
    hist_data_range: i64,
    underlying: Arc<impl HistoricalDataService + 'static + Send + Sync>,
) -> Arc<impl BacktestHistoricalDataManager> {
    Arc::new(implementation::BacktestHistoricalData {
        end,
        range,
        hist_data_range,
        underlying,
    })
}

mod implementation {
    use super::*;

    pub struct BacktestHistoricalData<H: HistoricalDataService + 'static + Send + Sync> {
        pub end: NaiveDate,
        pub range: i64,
        pub hist_data_range: i64,
        pub underlying: Arc<H>,
    }

    impl<H: HistoricalDataService + 'static + Send + Sync> BacktestHistoricalDataManager
        for BacktestHistoricalData<H>
    {
        fn all(&self) -> Arc<HashMap<String, Vec<Day>>> {
            self.underlying.fetch(self.end).clone()
        }
    }

    impl<H: HistoricalDataService + 'static + Send + Sync> HistoricalDataService
        for BacktestHistoricalData<H>
    {
        fn fetch(&self, end: NaiveDate) -> Arc<HashMap<String, Vec<Day>>> {
            // end is some past trading day. We want to fetch from end - range - hist_data_range to end
            let start = end - chrono::Duration::days(self.hist_data_range);
            println!(
                "BacktestHistoricalData.fetch: fetching from {} to {}",
                start, end
            );
            let data = self
                .underlying
                .fetch(end)
                .iter()
                .map(|(symbol, days)| {
                    let data = days
                        .iter()
                        .filter(|day| day.date >= start && day.date <= end)
                        .cloned()
                        .collect();
                    (symbol.clone(), data)
                })
                .collect();
            Arc::new(data)
        }
    }
}

// mod unused {
//     fn fetch(
//         &self,
//         symbol: &str,
//         start: NaiveDate,
//         end: NaiveDate,
//     ) -> reqwest::Result<Vec<&Day>> {
//         let symbol_history = self
//             .history
//             .get(symbol)
//             .expect(format!("No data for {}", symbol).as_ref());
//         let start_index = date_to_index(start, self.start);
//         let end_index = date_to_index(end, self.start);

//         println!(
//             "BacktestHistoricalData.fetch for {}: from {} to {}; indices {} - {}; symbol_history.len {}",
//             symbol,
//             start,
//             end,
//             start_index,
//             end_index,
//             symbol_history.len()
//         );

//         assert!(start_index >= 0);
//         assert!((end_index as usize) < symbol_history.len());
//         Ok(symbol_history[start_index as usize..end_index as usize].to_vec())
//     }

//     fn date_to_index(date: NaiveDate, start: NaiveDate) -> i64 {
//         assert!(date >= start);
//         (date - start).num_days()
//     }
// }

#[cfg(test)]
#[path = "./tests/backtest_historical_data_test.rs"]
mod backtest_historical_data_test;
#[path = "./tests/mock_historical_data_service.rs"]
mod mock_historical_data_service;
