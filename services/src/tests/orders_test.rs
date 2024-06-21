use crate::persistence;

use super::*;
use chrono::Local;

#[test]
fn test_create_order() {
    let access_token = std::env::var("TRADIER_SANDBOX_TOKEN").unwrap();
    let account_id = std::env::var("TRADIER_ACCOUNT_ID").unwrap();
    let persistence = persistence::new();
    let service = new(
        access_token,
        account_id,
        "sandbox.tradier.com".into(),
        persistence,
    )
    .expect("Failed to create OrdersService");
    let order = Order {
        tradier_id: None,
        date: Local::now().naive_local().date(),
        symbol: "SPY".to_string(),
        side: Side::Buy,
        quantity: 1,
    };

    match service.create_order(order) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("\n\n\nError: {}", e);
            assert!(false);
        }
    }
}
