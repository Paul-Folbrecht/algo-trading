use std::sync::Arc;

use crate::persistence::PersistenceService;
use core::http::*;
use domain::domain::*;
use std::{collections::HashMap, sync::Mutex};

pub trait OrderService {
    fn create_order(&self, order: Order) -> Result<Order, String>;
    fn get_position(&self, symbol: &str) -> Option<Position>;
}

pub fn new(
    access_token: String,
    account_id: String,
    base_url: String,
    persistence: Arc<impl PersistenceService + Send + Sync>,
) -> Result<Arc<impl OrderService>, String> {
    let positions = implementation::read_positions(&base_url, &access_token, &account_id)?;

    Ok(Arc::new(implementation::Orders {
        access_token,
        account_id,
        base_url,
        persistence,
        positions: Arc::new(Mutex::new(positions)),
    }))
}

mod implementation {
    use super::*;
    use chrono::Local;
    use serde::Deserialize;

    pub struct Orders<P: PersistenceService + Send + Sync> {
        pub access_token: String,
        pub account_id: String,
        pub base_url: String,
        pub persistence: Arc<P>,
        pub positions: Arc<Mutex<HashMap<String, Position>>>,
    }

    #[derive(Deserialize)]
    struct OrderResponse {
        order: OrderData,
    }

    #[derive(Deserialize)]
    struct OrderData {
        id: i64,
        status: String,
    }

    impl<P: PersistenceService + Send + Sync> OrderService for Orders<P> {
        fn create_order(&self, order: Order) -> Result<Order, String> {
            let url = format!(
                "https://{}/v1/accounts/{}/orders",
                self.base_url, self.account_id
            );
            let body = format!(
                "account_id={}&class=equity&symbol={}&side={}&quantity={}&type=market&duration=day",
                self.account_id, order.symbol, order.side, order.quantity
            );
            let response = post::<OrderResponse>(&url, &self.access_token, body);
            match response {
                Ok(response) => match response.order.status.as_str() {
                    "ok" => {
                        self.persistence.write(Box::new(order.clone()));
                        self.persistence
                            .write(Box::new(position_from(&order).clone()));
                        Ok(order.with_id(response.order.id))
                    }
                    _ => Err(response.order.status),
                },
                Err(e) => Err(e),
            }
        }

        fn get_position(&self, symbol: &str) -> Option<Position> {
            let positions = self.positions.lock().unwrap();
            positions.get(symbol).cloned()
        }
    }

    fn position_from(order: &Order) -> Position {
        Position {
            tradier_id: None,
            symbol: order.symbol.clone(),
            quantity: order.quantity,
            cost_basis: 0.0,
            date: Local::now(),
        }
    }

    #[derive(Deserialize)]
    struct PositionResponse {
        positions: Positions,
    }

    #[derive(Deserialize)]
    struct Positions {
        // position: Vec<TradierPosition>,
        position: TradierPosition,
    }

    pub fn read_positions(
        base_url: &str,
        access_token: &str,
        account_id: &str,
    ) -> Result<HashMap<String, Position>, String> {
        let url = format!("https://{}/v1/accounts/{}/positions", base_url, account_id);
        println!("url: {}", url);
        let response = get::<PositionResponse>(&url, &access_token);

        match response {
            Ok(response) => {
                let mut positions = HashMap::new();
                // response
                //     .positions
                //     .position
                //     .into_iter()
                //     .for_each(|position| {
                //         positions.insert(position.symbol.clone(), position.into());
                //     });
                let position = response.positions.position;
                positions.insert(position.symbol.clone(), position.into());
                Ok(positions)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
#[path = "./tests/orders_test.rs"]
mod orders_test;
