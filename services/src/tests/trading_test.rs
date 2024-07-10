use super::*;
use chrono::{Local, NaiveDate};
use domain::domain::Day;
use implementation::*;

struct MockHistoricalDataService {}
impl HistoricalDataService for MockHistoricalDataService {
    fn fetch(&self, _: &str, _: NaiveDate, _: NaiveDate) -> Result<Vec<Day>, reqwest::Error> {
        Ok(vec![
            Day {
                symbol: Some("SPY".to_string()),
                date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 10.0,
                volume: 1,
            },
            Day {
                symbol: Some("SPY".to_string()),
                date: NaiveDate::from_ymd_opt(2024, 4, 2).unwrap(),
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 10.0,
                volume: 2,
            },
            Day {
                symbol: Some("SPY".to_string()),
                date: NaiveDate::from_ymd_opt(2024, 4, 3).unwrap(),
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 20.0,
                volume: 3,
            },
        ])
    }
}

#[test]
fn test_load_history() {
    let symbols = vec!["SPY".to_string()];
    let historical_data_service = Arc::new(MockHistoricalDataService {});
    let date = Local::now().naive_local().date();
    let data = load_history(date, &symbols, historical_data_service);
    let spy = data.get(&"SPY".to_string()).unwrap();
    assert_eq!(spy.mean, 13.333333333333334);
    assert_eq!(spy.std_dev, 4.714045207910316);
}

struct MockOrderService {}
impl OrderService for MockOrderService {
    fn create_order(&self, order: Order) -> Result<Order, String> {
        Ok(order.with_id(1000))
    }

    fn get_position(&self, symbol: &str) -> Option<Position> {
        match symbol {
            "SPY" => Some(Position {
                symbol: symbol.to_string(),
                quantity: 100,
                broker_id: None,
                cost_basis: 10000.0,
                date: Local::now(),
            }),
            "AMZN" => None,
            _ => None,
        }
    }

    fn update_position(&self, position: &Position) {
        unimplemented!()
    }
}

#[test]
fn test_handle_quote() {
    let date = Local::now().naive_local().date();
    let orders = Arc::new(MockOrderService {});

    let quote = Quote {
        symbol: "SPY".to_string(),
        bid: 80.0,
        ask: 80.0,
        biddate: Local::now(),
        askdate: Local::now(),
    };

    match maybe_create_order(date, Signal::Buy, orders.get_position("SPY"), &quote, 10000) {
        Some(order) => {
            assert_eq!(order.symbol, "SPY");
            // Capital of $10K - 100 shares * 80 = $2000 remaining capital = 25 shares at $80
            assert_eq!(order.quantity, 25);
            assert_eq!(order.side, Side::Buy);
        }
        None => panic!("Expected an order"),
    }

    match maybe_create_order(
        date,
        Signal::Sell,
        orders.get_position("SPY"),
        &quote,
        10000,
    ) {
        Some(order) => {
            assert_eq!(order.symbol, "SPY");
            // We always unwind completely and have 100 shares, so any Sell signal should sell all
            assert_eq!(order.quantity, 100);
            assert_eq!(order.side, Side::Sell);
        }
        None => panic!("Expected an order"),
    }

    match maybe_create_order(
        date,
        Signal::None,
        orders.get_position("SPY"),
        &quote,
        10000,
    ) {
        Some(_) => panic!("Expected no order"),
        None => {}
    }
}
