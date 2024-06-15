use chrono::{DateTime, Local, NaiveDate};
use core::serde::{tradier_date_format, tradier_date_time_format};
use serde::Deserialize;
use std::fmt::{Display, Formatter};

use crate::serde::side_format;

#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    #[serde(with = "tradier_date_time_format")]
    pub biddate: DateTime<Local>,
    #[serde(with = "tradier_date_time_format")]
    pub askdate: DateTime<Local>,
}

#[derive(Deserialize, Debug)]
pub struct History {
    pub day: Vec<Day>,
}

#[derive(Deserialize, Debug)]
pub struct Day {
    #[serde(with = "tradier_date_format")]
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug)]
pub struct SymbolData {
    pub symbol: String,
    pub history: Vec<Day>,
    pub mean: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub tradier_id: Option<i64>,
    #[serde(with = "tradier_date_format")]
    pub date: NaiveDate,
    pub symbol: String,
    #[serde(with = "side_format")]
    pub side: Side,
    pub qty: i64,
    pub price: f64,
}

impl Order {
    pub fn with_id(&self, tradier_id: i64) -> Self {
        Order {
            tradier_id: Some(tradier_id),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Strategy {
    MeanReversion { symbols: Vec<String> },
}

pub trait StrategyHandler {
    fn handle(&self, quote: &Quote, data: &SymbolData);
}

impl Strategy {
    pub fn new(name: &str, symbols: Vec<String>) -> Self {
        match name {
            "mean-reversion" => Strategy::MeanReversion { symbols },
            _ => panic!("Unknown strategy: {}", name),
        }
    }
}

impl StrategyHandler for Strategy {
    fn handle(&self, quote: &Quote, data: &SymbolData) {
        match self {
            Strategy::MeanReversion { symbols } => {
                if symbols.contains(&quote.symbol) {
                    println!("MeanReversionStrategy handling quote: {:?}", quote);
                    println!(
                        "Px: {}; Mean: {}; Std Dev: {}",
                        quote.ask, data.mean, data.std_dev
                    );
                    println!(
                        "quote.ask: {}; (data.mean - 2.0 * data.std_dev): {}",
                        quote.ask,
                        data.mean - 2.0 * data.std_dev
                    );

                    let buy = quote.ask < data.mean - 2.0 * data.std_dev;
                    let sell = quote.ask > data.mean + 2.0 * data.std_dev;

                    // - Buy:
                    //   - If position qty < target_position_qty, buy the difference
                    //   - Else log
                    // - Sell:
                    //   - If we have a position, unwind
                    //   - Else log
                    // - target_position_qty:
                    //   - Config capital per symbol
                    if buy {
                        println!("***Buy signal for {}***", quote.symbol);
                    } else {
                        println!("No signal for {}", quote.symbol);
                    }
                }
            }
        }
    }
}
