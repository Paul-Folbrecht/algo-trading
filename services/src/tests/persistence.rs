use std::{thread, time::Duration};

use super::*;
use chrono::NaiveDate;
use domain::domain::Side::Buy;

#[test]
fn test_order_persistence() {
    let order = Order {
        tradier_id: Some(0),
        date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        side: Buy,
        symbol: "AAPL".to_string(),
        qty: 100,
    };

    let db = persistence::new();
    let shutdown = Arc::new(AtomicBool::new(false));
    db.init(shutdown.clone()).expect("Persistence init failed");
    let result = db.write(Box::new(order)).expect("Write order failed");
    assert_eq!(result, ());
    thread::sleep(Duration::from_secs(1));
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    // Now go to the mongo shell and verify the order, homey
    // And yes this is an integration test, not a unit test
}
