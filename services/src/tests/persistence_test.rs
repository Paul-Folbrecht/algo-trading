use super::*;
use crate::persistence;
use chrono::{Local, NaiveDate};
use domain::domain::Side::Buy;
use std::{thread, time::Duration};

#[test]
fn test_persistence() {
    let order = Order {
        tradier_id: Some(0),
        date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        side: Buy,
        symbol: "AAPL".to_string(),
        quantity: 100,
    };

    let position = Position {
        tradier_id: Some(0),
        symbol: "AAPL".to_string(),
        quantity: 100,
        cost_basis: 1000.0,
        date: Local::now(),
    };

    let db = persistence::new();
    let shutdown = Arc::new(AtomicBool::new(false));
    db.init(shutdown.clone()).expect("Persistence init failed");
    db.write(Box::new(order)).expect("Write order failed");
    db.write(Box::new(position)).expect("Write position failed");
    thread::sleep(Duration::from_secs(1));
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    thread::sleep(Duration::from_secs(1));
    // Now go to the mongo shell and verify the order, homey
    // And yes this is an integration test, not a unit test
}
