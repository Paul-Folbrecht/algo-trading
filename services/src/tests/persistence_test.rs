use super::*;
use crate::persistence;
use chrono::{Local, NaiveDate};
use domain::domain::{RealizedPnL, Side::Buy};
use std::{thread, time::Duration};

#[test]
fn x() {
    let order = Order {
        id: Some(0),
        date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        side: Buy,
        symbol: "SPY".to_string(),
        quantity: 100,
        px: Some(100.0),
    };

    let position = Position {
        broker_id: Some(0),
        symbol: "SPY".to_string(),
        quantity: 100,
        cost_basis: 1000.0,
        date: Local::now(),
    };

    let pnl = RealizedPnL {
        id: 1000,
        symbol: "SPY".to_string(),
        date: Local::now(),
        pnl: 100.0,
        strategy: "mean-reversion".to_string(),
    };

    let db = persistence::new("mongodb://localhost:27017".to_string());
    let shutdown = Arc::new(AtomicBool::new(false));
    db.init(shutdown.clone()).expect("Persistence init failed");
    db.write(Box::new(order)).expect("Write order failed");
    db.write(Box::new(position)).expect("Write position failed");
    db.write(Box::new(pnl)).expect("Write pnl failed");
    thread::sleep(Duration::from_secs(1));
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    thread::sleep(Duration::from_secs(1));
    // Now go to the mongo shell and verify the writes
    // And yes this is an integration test, not a unit test
}
