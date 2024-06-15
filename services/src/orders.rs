use std::sync::Arc;

use backoff::{retry, Error, ExponentialBackoff};
use domain::domain::*;

pub trait OrderService {
    fn create_order(&self, order: Order) -> Result<Order, String>;
}

pub fn new(access_token: String, account_id: String) -> Arc<impl OrderService> {
    Arc::new(implementation::Orders {
        access_token,
        account_id,
    })
}

mod implementation {
    use serde::Deserialize;

    use super::*;

    pub struct Orders {
        pub access_token: String,
        pub account_id: String,
    }

    #[derive(Deserialize)]
    struct OrderResponse {
        id: i64,
        status: String,
    }

    impl OrderService for Orders {
        fn create_order(&self, order: Order) -> Result<Order, String> {
            let op = || {
                let url = format!("https://api.com/v1/accounts/{}/orders", self.account_id);
                reqwest::blocking::Client::new()
                    .post(url)
                    .send()
                    .map_err(backoff::Error::transient)
            };

            let response = retry(ExponentialBackoff::default(), op)
                .map_err::<String, _>(|e| e.to_string())
                .and_then(|r| {
                    r.json::<OrderResponse>()
                        .map_err::<String, _>(|e| e.to_string())
                });

            Ok(order.with_id(response.unwrap().id))
        }
    }
}

#[cfg(test)]
#[path = "./tests/orders_test.rs"]
mod orders_test;
