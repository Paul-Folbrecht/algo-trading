use crate::backtest_historical_data::BacktestHistoricalDataManager;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use core::util::print_map;
use domain::domain::{Day, Quote};
use log::*;
use services::market_data::MarketDataService;
use std::{collections::HashMap, sync::Arc};

pub trait BacktestMarketDataManager {
    fn service_for_date(
        &self,
        date: NaiveDate,
    ) -> Result<Arc<impl MarketDataService + 'static + Send + Sync>, String>;
}

pub fn new(
    access_token: String,
    symbols: Vec<String>,
    backtest_range: i64,
    end: NaiveDate,
    backtest_historical_data: Arc<impl BacktestHistoricalDataManager + Send + Sync>,
) -> Arc<impl BacktestMarketDataManager> {
    // We need to turn a map of symbol->days into a map of date->quotes
    let history: Vec<Day> = backtest_historical_data
        .all()
        .values()
        .flatten()
        .cloned()
        .collect();
    info!("\n\nBacktestMarketDataManager: history:\n{:?}", history);

    let mut quotes: HashMap<NaiveDate, Vec<Quote>> = HashMap::new();
    for day in history {
        let date: DateTime<Local> = day
            .date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .earliest()
            .expect("Failed to convert date to datetime");
        quotes.entry(day.date).or_default().push(Quote {
            symbol: day.symbol.expect("Missing symbol").clone(),
            bid: day.close,
            ask: day.close,
            biddate: date,
            askdate: date,
        })
    }
    print_map("Quotes", &quotes);

    Arc::new(implementation::BacktestMarketData { quotes })
}

mod implementation {
    use super::*;
    use chrono::NaiveDate;
    use crossbeam_channel::Receiver;
    use std::collections::HashMap;

    pub struct BacktestMarketData {
        pub quotes: HashMap<NaiveDate, Vec<Quote>>,
    }

    impl BacktestMarketDataManager for BacktestMarketData {
        fn service_for_date(
            &self,
            date: NaiveDate,
        ) -> Result<Arc<impl MarketDataService + 'static + Send + Sync>, String> {
            match self.quotes.get(&date) {
                Some(quotes) => Ok(Arc::new(BacktestMarketDataService {
                    quotes: quotes.clone(),
                })),
                None => Err(format!("No quotes for date {}", date)),
            }
        }
    }

    pub struct BacktestMarketDataService {
        quotes: Vec<Quote>,
    }

    impl MarketDataService for BacktestMarketDataService {
        fn init(
            &self,
            shutdown: Arc<std::sync::atomic::AtomicBool>,
            symbols: Vec<String>,
        ) -> Result<std::thread::JoinHandle<()>, String> {
            Err("Please don't call this method".to_string())
        }

        fn subscribe(&self) -> Result<Receiver<Quote>, String> {
            let (sender, receiver) = crossbeam_channel::unbounded();
            self.quotes
                .iter()
                .for_each(|quote| match sender.send(quote.clone()) {
                    Ok(_) => (),
                    Err(e) => info!("Error sending quote to subscriber: {}", e),
                });
            Ok(receiver)
        }

        fn unsubscribe(&self, subscriber: &Receiver<Quote>) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "./tests/backtest_market_data_manager_test.rs"]
mod backtest_market_data_manager_test;
#[path = "./tests/mock_historical_data_service.rs"]
mod mock_historical_data_service;
