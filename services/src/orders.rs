use std::sync::Arc;

use crate::persistence::PersistenceService;
use core::http::*;
use domain::domain::*;
use std::{collections::HashMap, sync::Mutex};

pub trait OrderService {
    fn create_order(&self, order: Order, strategy: String) -> Result<Order, String>;
    fn get_position(&self, symbol: &str) -> Option<Position>;
    fn update_position(&self, position: &Position);
}

pub fn new(
    access_token: String,
    account_id: String,
    base_url: String,
    persistence: Arc<impl PersistenceService + Send + Sync>,
) -> Result<Arc<impl OrderService>, String> {
    let positions = implementation::read_positions(&base_url, &access_token, &account_id)?;
    println!("Read positions from broker:\n{:?}", positions);
    implementation::update_local_positions(persistence.clone(), &positions)?;

    Ok(Arc::new(implementation::Orders {
        access_token,
        account_id,
        base_url,
        persistence,
        positions: Arc::new(Mutex::new(positions)),
    }))
}

pub mod implementation {
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

    #[derive(Deserialize, Debug)]
    struct OrderResponse {
        order: OrderData,
    }

    #[derive(Deserialize, Debug)]
    struct OrderData {
        id: i64,
        status: String,
    }

    impl<P: PersistenceService + Send + Sync> OrderService for Orders<P> {
        fn create_order(&self, order: Order, strategy: String) -> Result<Order, String> {
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
                        println!("Response: {:?}", response);
                        let new_order = order.with_id(response.order.id);
                        match self.persistence.write(Box::new(new_order.clone())) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Error writing order: {}", e),
                        }

                        let position = position_from(&new_order, self.get_position(&order.symbol));
                        match self.persistence.write(Box::new(position.clone())) {
                            Ok(_) => self.update_position(&position),
                            Err(e) => eprintln!("Error writing position: {}", e),
                        }

                        if order.side == Side::Sell {
                            let pnl = calc_pnl(position, &order, strategy);
                            match self.persistence.write(Box::new(pnl.clone())) {
                                Ok(_) => println!("Generated P&L: {:?}", pnl),
                                Err(e) => eprintln!("Error writing position: {}", e),
                            }
                        }

                        Ok(new_order)
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

        fn update_position(&self, position: &Position) {
            self.positions
                .lock()
                .unwrap()
                .insert(position.symbol.clone(), position.clone());
        }
    }

    pub fn position_from(order: &Order, existing: Option<Position>) -> Position {
        match order.side {
            Side::Buy => position_from_buy(order, existing),
            Side::Sell => position_from_sell(order, existing),
        }
    }

    pub fn position_from_buy(order: &Order, existing: Option<Position>) -> Position {
        match existing {
            Some(position) => Position {
                quantity: position.quantity + order.quantity,
                ..position
            },
            None => {
                Position {
                    // broker_id & cost_basis will be updated when positions are read from the broker
                    // These fields are not relevant to trading
                    broker_id: None,
                    symbol: order.symbol.clone(),
                    quantity: order.quantity,
                    cost_basis: order.px.unwrap_or(0.0) * order.quantity as f64, // Estimate
                    date: Local::now(),
                }
            }
        }
    }

    pub fn position_from_sell(order: &Order, existing: Option<Position>) -> Position {
        match existing {
            Some(position) => {
                assert!(
                    order.quantity <= position.quantity,
                    "Attempted invalid unwind"
                );
                Position {
                    quantity: 0,
                    ..position
                }
            }
            None => panic!("Attempted unwind with no position: {:?}", order),
        }
    }

    pub fn calc_pnl(position: Position, order: &Order, strategy: String) -> RealizedPnL {
        let price = order.px.unwrap_or(0.0);
        let proceeds = price * order.quantity as f64;
        let pnl = proceeds - position.cost_basis;

        println!(
            "Calced Realized P&L; proceeds: {}; cost basis: {}; pnl: {}; price: {}; quantity: {}",
            proceeds, position.cost_basis, pnl, price, order.quantity
        );
        RealizedPnL {
            id: order.id(),
            symbol: order.symbol.clone(),
            date: order.date,
            pnl,
            strategy: strategy.to_string(),
        }
    }

    #[derive(Deserialize)]
    struct PositionResponse {
        positions: Positions,
    }

    #[derive(Deserialize)]
    struct Positions {
        position: Vec<TradierPosition>,
    }

    pub fn read_positions(
        base_url: &str,
        access_token: &str,
        account_id: &str,
    ) -> Result<HashMap<String, Position>, String> {
        let url = format!("https://{}/v1/accounts/{}/positions", base_url, account_id);
        println!("url: {}", url);
        let response = get::<PositionResponse>(&url, access_token);

        match response {
            Ok(response) => {
                let mut positions = HashMap::new();
                response
                    .positions
                    .position
                    .into_iter()
                    .for_each(|position| {
                        positions.insert(position.symbol.clone(), position.into());
                    });
                Ok(positions)
            }
            Err(e) => {
                eprintln!("Error reading positions - probably there are < 2: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    pub fn update_local_positions(
        persistence: Arc<impl PersistenceService>,
        positions: &HashMap<String, Position>,
    ) -> Result<(), String> {
        // In the future, we may rec, but for now we'll update all positions from the source of truth
        persistence.drop_positions()?;
        positions
            .values()
            .try_for_each(|position| persistence.write(Box::new(position.clone())))
    }
}

#[cfg(test)]
#[path = "./tests/orders_test.rs"]
mod orders_test;
