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
    use services::trading::{self, TradingService};

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
        //   - Construct BacktestMarketDataService from MarketDataManager data; use single BacktestHistoricalDataService
        //   - run() strategies - will subscribe to MarketDataService and be fed quotes
        // - Report Realized PnL, open positions from OrderService
        fn run(&self) -> Result<(), String> {
            let start = self.end - chrono::Duration::days(self.backtest_range);
            println!("Running backtest from {} to {}", start, self.end);

            for i in 0..=self.backtest_range {
                let date = start + chrono::Duration::days(i);

                println!("\nRunning for {}", date);
                match self.market_data_manager.service_for_date(date) {
                    Ok(market_data) => {
                        self.strategies.clone().into_iter().for_each(|strategy| {
                            let mut trading_service = trading::new(
                                date,
                                strategy.name.clone(),
                                strategy.symbols.clone(),
                                strategy.capital.clone(),
                                market_data.clone(),
                                self.historical_data.clone(),
                                self.orders.clone(),
                            );

                            match trading_service.run() {
                                Ok(_) => {
                                    println!(
                                        "Strategy '{}' ran successfully for {}",
                                        strategy.name, date
                                    );
                                    trading_service
                                        .shutdown()
                                        .expect("Unexpected error shutting down trading_service");
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Error starting TradingService {}: {}",
                                        strategy.name, e
                                    )
                                }
                            }
                        });
                    }
                    Err(_) => {
                        eprintln!("Skipping {} - no data (weekend or holiday)", date);
                        continue;
                    }
                }
            }

            Ok(()) // @todo return Pnl & positions
        }
    }
}
