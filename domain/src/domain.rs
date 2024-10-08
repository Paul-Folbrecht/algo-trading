use chrono::{DateTime, Local, NaiveDate};
use core::serde::{millis_date_time_format, rfc_3339_date_time_format, string_date_format};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    fmt::{self, Display, Formatter},
};

use crate::serde::side_format;

#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    #[serde(with = "millis_date_time_format")]
    pub biddate: DateTime<Local>,
    #[serde(with = "millis_date_time_format")]
    pub askdate: DateTime<Local>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Day {
    pub symbol: Option<String>,
    #[serde(with = "string_date_format")]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub trait Persistable {
    fn as_any(&self) -> &dyn Any;
    fn id(&self) -> i64;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: Option<i64>,
    #[serde(with = "string_date_format")]
    pub date: NaiveDate,
    pub symbol: String,
    #[serde(with = "side_format")]
    pub side: Side,
    // Integer quantity as we'll only trade equities
    pub quantity: i64,
    pub px: Option<f64>,
}

impl Persistable for Order {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> i64 {
        self.id.unwrap_or(0)
    }
}

impl Order {
    pub fn with_id(&self, id: i64) -> Self {
        Order {
            id: Some(id),
            ..self.clone()
        }
    }
}

#[derive(Deserialize)]
pub struct TradierPosition {
    pub id: i64,
    pub symbol: String,
    pub quantity: f64,
    pub cost_basis: f64,
    #[serde(with = "rfc_3339_date_time_format")]
    pub date_acquired: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub broker_id: Option<i64>,
    pub symbol: String,
    // Integer quantity as we'll only trade equities
    pub quantity: i64,
    pub cost_basis: f64,
    #[serde(with = "millis_date_time_format")]
    pub date: DateTime<Local>,
}

impl From<TradierPosition> for Position {
    fn from(tp: TradierPosition) -> Self {
        Position {
            broker_id: Some(tp.id),
            symbol: tp.symbol,
            quantity: tp.quantity as i64,
            cost_basis: tp.cost_basis,
            date: tp.date_acquired,
        }
    }
}

impl Position {
    pub fn with_id(&self, id: i64) -> Self {
        Position {
            broker_id: Some(id),
            ..self.clone()
        }
    }

    pub fn with_cost_basis(&self, cost_basis: f64) -> Self {
        Position {
            cost_basis,
            ..self.clone()
        }
    }
}

impl Persistable for Position {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> i64 {
        self.broker_id.unwrap_or(0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RealizedPnL {
    pub id: i64,
    pub symbol: String,
    #[serde(with = "string_date_format")]
    pub date: NaiveDate,
    pub pnl: f64,
    pub strategy: String,
}

impl Persistable for RealizedPnL {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    None,
}

#[derive(Debug, Clone)]
pub enum Strategy {
    MeanReversion { symbols: Vec<String> },
}

impl Strategy {
    pub fn new(name: &str, symbols: Vec<String>) -> Self {
        match name {
            "mean-reversion" => Strategy::MeanReversion { symbols },
            _ => panic!("Unknown strategy: {}", name),
        }
    }
}

impl Display for Strategy {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait StrategyHandler {
    fn handle(&self, quote: &Quote, data: &SymbolData) -> Result<Signal, String>;
}

impl StrategyHandler for Strategy {
    fn handle(&self, quote: &Quote, data: &SymbolData) -> Result<Signal, String> {
        // let quote = if _quote.symbol == "AAPL" {
        //     Quote {
        //         symbol: "AAPL".to_string(),
        //         bid: 100.0,
        //         ask: 100.0,
        //         biddate: Local::now(),
        //         askdate: Local::now(),
        //     }
        // } else {
        //     _quote.clone()
        // };
        match self {
            Strategy::MeanReversion { symbols } => {
                if symbols.contains(&quote.symbol) {
                    info!("MeanReversionStrategy handling quote: {:?}", quote);
                    info!(
                        "Px: {}; Mean: {}; Std Dev: {}",
                        quote.ask, data.mean, data.std_dev
                    );
                    info!(
                        "quote.ask: {}; (mean - 2.0 * std_dev): {}",
                        quote.ask,
                        data.mean - 2.0 * data.std_dev
                    );

                    let buy = quote.ask < data.mean - 2.0 * data.std_dev;
                    let sell = quote.ask > data.mean + 2.0 * data.std_dev;

                    if buy {
                        info!("***Buy signal for {}***", quote.symbol);
                        Ok(Signal::Buy)
                    } else if sell {
                        info!("***Sell signal for {}***", quote.symbol);
                        Ok(Signal::Sell)
                    } else {
                        info!("No signal for {}", quote.symbol);
                        Ok(Signal::None)
                    }
                } else {
                    info!("Symbol {} not in strategy", quote.symbol);
                    Ok(Signal::None)
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./domain_test.rs"]
mod domain_test;
