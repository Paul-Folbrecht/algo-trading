use super::*;
use crate::persistence;
use chrono::Local;

#[test]
fn test_create_order() {
    let access_token = std::env::var("TRADIER_SANDBOX_TOKEN").unwrap();
    let account_id = std::env::var("TRADIER_ACCOUNT_ID").unwrap();
    let persistence = persistence::new("mongodb://localhost:27017".to_string());
    let service = new(
        access_token,
        account_id,
        "sandbox.tradier.com".into(),
        persistence,
    )
    .expect("Failed to create OrdersService");
    let order = Order {
        broker_id: None,
        date: Local::now().naive_local().date(),
        symbol: "SPY".to_string(),
        side: Side::Buy,
        quantity: 1,
    };

    match service.create_order(order.clone()) {
        Ok(_) => println!("Order created successfully: {:?}", order),
        Err(e) => {
            eprintln!("\n\n\nError: {}", e);
            assert!(false);
        }
    }
}
