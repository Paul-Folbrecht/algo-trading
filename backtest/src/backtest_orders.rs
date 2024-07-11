use services::orders::OrderService;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

pub fn new() -> Arc<impl OrderService + Send + Sync> {
    Arc::new(implementation::BacktestOrders {
        positions: Arc::new(Mutex::new(HashMap::new())),
        pnl: Arc::new(Mutex::new(Vec::new())),
    })
}

mod implementation {
    use super::*;
    use domain::domain::*;
    use services::orders::implementation::*;

    pub struct BacktestOrders {
        pub positions: Arc<Mutex<HashMap<String, Position>>>,
        pub pnl: Arc<Mutex<Vec<RealizedPnL>>>,
    }

    impl OrderService for BacktestOrders {
        fn create_order(&self, order: Order, strategy: String) -> Result<Order, String> {
            let position = position_from(&order, self.get_position(&order.symbol));
            self.update_position(&position);

            if order.side == Side::Sell {
                let pnl = calc_pnl(position, &order, strategy);
                self.pnl.lock().unwrap().push(pnl.clone());
                println!("Generated P&L: {:?}", pnl);
            }

            Ok(order)
        }

        fn get_position(&self, symbol: &str) -> Option<Position> {
            let positions = self.positions.lock().unwrap();
            positions.get(symbol).cloned()
        }

        fn update_position(&self, position: &Position) {
            self.positions
                .lock()
                .unwrap()
                .insert(position.symbol.clone(), position.clone());
        }
    }
}
