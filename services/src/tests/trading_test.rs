use super::*;
use chrono::{Local, NaiveDate};
use domain::domain::{Day, History};
use implementation::*;

struct MockHistoricalDataService {}
impl HistoricalDataService for MockHistoricalDataService {
    fn fetch(&self, _: &str, _: NaiveDate, _: NaiveDate) -> Result<History, reqwest::Error> {
        Ok(History {
            day: vec![
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
                    open: 1.0,
                    high: 1.0,
                    low: 1.0,
                    close: 10.0,
                    volume: 1,
                },
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 2).unwrap(),
                    open: 2.0,
                    high: 2.0,
                    low: 2.0,
                    close: 10.0,
                    volume: 2,
                },
                Day {
                    date: NaiveDate::from_ymd_opt(2024, 4, 3).unwrap(),
                    open: 3.0,
                    high: 3.0,
                    low: 3.0,
                    close: 20.0,
                    volume: 3,
                },
            ],
        })
    }
}

#[test]
fn test_load_history() {
    let symbols = vec!["AAPL".to_string()];
    let historical_data_service = Arc::new(MockHistoricalDataService {});
    let data = load_history(&symbols, historical_data_service);
    let aapl = data.get(&"AAPL".to_string()).unwrap();
    assert_eq!(aapl.mean, 13.333333333333334);
    assert_eq!(aapl.std_dev, 4.714045207910316);
}

struct MockOrderService {}
impl OrderService for MockOrderService {
    fn create_order(&self, order: Order) -> Result<Order, String> {
        Ok(order.with_id(1000))
    }

    fn get_position(&self, symbol: &str) -> Option<Position> {
        match symbol {
            "AAPL" => Some(Position {
                symbol: symbol.to_string(),
                quantity: 100,
                tradier_id: None,
                cost_basis: 10000.0,
                date: Local::now(),
            }),
            "AMZN" => None,
            _ => None,
        }
    }
}

#[test]
fn test_handle_quote() {
    let orders = Arc::new(MockOrderService {});

    let quote = Quote {
        symbol: "AAPL".to_string(),
        bid: 80.0,
        ask: 80.0,
        biddate: Local::now(),
        askdate: Local::now(),
    };

    match maybe_create_order(Signal::Buy, orders.get_position("AAPL"), &quote, 10000) {
        Some(order) => {
            assert_eq!(order.symbol, "AAPL");
            // Capital of $10K - 100 shares * 80 = $2000 remaining capital = 25 shares at $80
            assert_eq!(order.quantity, 25);
            assert_eq!(order.side, Side::Buy);
        }
        None => panic!("Expected an order"),
    }

    match maybe_create_order(Signal::Sell, orders.get_position("AAPL"), &quote, 10000) {
        Some(order) => {
            assert_eq!(order.symbol, "AAPL");
            // We always unwind completely and have 100 shares, so any Sell signal should sell all
            assert_eq!(order.quantity, 100);
            assert_eq!(order.side, Side::Sell);
        }
        None => panic!("Expected an order"),
    }

    match maybe_create_order(Signal::None, orders.get_position("AAPL"), &quote, 10000) {
        Some(_) => panic!("Expected no order"),
        None => {}
    }
}
