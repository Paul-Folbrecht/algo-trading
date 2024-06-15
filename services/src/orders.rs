use std::sync::Arc;

use backoff::{retry, ExponentialBackoff};
use domain::domain::*;

pub trait OrderService {
    fn create_order(&self, order: Order) -> Result<Order, String>;
}

pub fn new(access_token: String, account_id: String, sandbox: bool) -> Arc<impl OrderService> {
    Arc::new(implementation::Orders {
        access_token,
        account_id,
        sandbox,
    })
}

mod implementation {
    use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
    use serde::Deserialize;

    use super::*;

    pub struct Orders {
        pub access_token: String,
        pub account_id: String,
        pub sandbox: bool,
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

    impl OrderService for Orders {
        fn create_order(&self, order: Order) -> Result<Order, String> {
            let op = || {
                let base = if self.sandbox {
                    "sandbox.tradier.com"
                } else {
                    "api.tradier.com"
                };
                let url = format!("https://{}/v1/accounts/{}/orders", base, self.account_id);
                let body = format!("account_id={}&class=equity&symbol={}&side={}&quantity={}&type=market&duration=day",
                    self.account_id, order.symbol, order.side, order.qty);
                reqwest::blocking::Client::new()
                    .post(url)
                    .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
                    .header(ACCEPT, "application/json")
                    .header(CONTENT_LENGTH, "0")
                    .body(body)
                    .send()
                    .map_err(backoff::Error::transient)
            };

            let backoff = ExponentialBackoff {
                initial_interval: std::time::Duration::from_millis(100),
                max_interval: std::time::Duration::from_secs(2),
                max_elapsed_time: Some(std::time::Duration::from_secs(5)),
                ..ExponentialBackoff::default()
            };
            let response: Result<OrderResponse, String> = retry(backoff, op)
                .map_err::<String, _>(|e| e.to_string())
                .and_then(|r| {
                    // println!("Response: {:?}", r.text().unwrap());
                    // return Err(String::from("Not implemented"));
                    r.json::<OrderResponse>()
                        .map_err::<String, _>(|e| format!("Could not parse body - note, this will occur in any error condition: {}", e.to_string()))
                });

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
