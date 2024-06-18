use std::sync::Arc;

use crate::persistence::PersistenceService;
use core::http::*;
use domain::domain::*;

pub trait OrderService {
    fn create_order(&self, order: Order) -> Result<Order, String>;
}

pub fn new(
    access_token: String,
    account_id: String,
    sandbox: bool,
    persistence: Arc<impl PersistenceService + Send + Sync>,
) -> Arc<impl OrderService> {
    Arc::new(implementation::Orders {
        access_token,
        account_id,
        sandbox,
        base_url: if sandbox {
            "sandbox.tradier.com".into()
        } else {
            "api.tradier.com".into()
        },
        persistence,
    })
}

mod implementation {
    use serde::Deserialize;

    use super::*;

    pub struct Orders<P: PersistenceService + Send + Sync> {
        pub access_token: String,
        pub account_id: String,
        pub sandbox: bool,
        pub base_url: String,
        pub persistence: Arc<P>,
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
                self.account_id, order.symbol, order.side, order.qty
            );

            let response: Result<OrderResponse, String> =
                post::<OrderResponse>(&url, &self.access_token, body);

            match response {
                Ok(response) => match response.order.status.as_str() {
                    "ok" => Ok(order.with_id(response.order.id)),
                    _ => Err(response.order.status),
                },
                Err(e) => Err(e),
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/orders_test.rs"]
mod orders_test;
