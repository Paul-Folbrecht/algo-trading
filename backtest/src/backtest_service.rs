use crate::backtest_market_data_manager::BacktestMarketDataManager;
use app_config::app_config::Strategy;
use chrono::NaiveDate;
use services::{historical_data::HistoricalDataService, orders::OrderService};
use std::sync::Arc;

pub trait BacktestService {
    fn run(&self) -> Result<(), String>;
}

pub fn new(
    end: NaiveDate,
    backtest_range: i64,
    historical_data: Arc<impl HistoricalDataService + 'static + Send + Sync>,
    market_data_manager: Arc<impl BacktestMarketDataManager + 'static + Send + Sync>,
    orders: Arc<impl OrderService + 'static + Send + Sync>,
    strategies: Vec<Strategy>,
) -> Arc<impl BacktestService + Send + Sync> {
    Arc::new(implementation::Backtest {
        end,
        backtest_range,
        historical_data,
        market_data_manager,
        orders,
        strategies,
    })
}

mod implementation {
    use super::*;

    pub struct Backtest<
        H: HistoricalDataService + 'static + Send + Sync,
        M: BacktestMarketDataManager + 'static + Send + Sync,
        O: OrderService + 'static + Send + Sync,
    > {
        pub end: NaiveDate,
        pub backtest_range: i64,
        pub historical_data: Arc<H>,
        pub market_data_manager: Arc<M>,
        pub orders: Arc<O>,
        pub strategies: Vec<Strategy>,
    }

    impl<
            H: HistoricalDataService + Send + Sync,
            M: BacktestMarketDataManager + Send + Sync,
            O: OrderService + Send + Sync,
        > BacktestService for Backtest<H, M, O>
    {
        // - For each date in range:
        //     - Construct BacktestMarketDataService from MarketDataManager data; use single BacktestHistoricalDataService
        //     - run() strategies - will subscribe to MarketDataService and be fed quotes
        //   - Report Realized PnL, open positions from OrderService
        fn run(&self) -> Result<(), String> {
            println!("Running backtest");
            let start = self.end - chrono::Duration::days(self.backtest_range);
            //let mut days = Vec::with_capacity((backtest_range + hist_data_range) as usize);
            for i in 0..=self.backtest_range {
                let date = start + chrono::Duration::days(i);
                let market_data = self.market_data_manager.service_for_date(date);
            }
            // let orders: impl OrderService = todo!();
            // let trading_service = self.strategies.into_iter().for_each(|strategy| {
            //     let date = self.end;
            //     let market_data = self.market_data_manager.service_for_date(date);
            //     let mut trading_service = trading::new(
            //         date,
            //         strategy.name.clone(),
            //         strategy.symbols.clone(),
            //         strategy.capital.clone(),
            //         market_data.clone(),
            //         self.historical_data,
            //         orders,
            //     );
            // });
            Ok(())
        }
    }
}
