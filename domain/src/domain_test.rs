use super::*;

#[test]
fn test_mean_reversion_strategy() {
    let strategy = Strategy::new("mean-reversion", vec!["SPY".to_string()]);
    let symbol_data = SymbolData {
        mean: 100.0,
        std_dev: 4.714045207910316,
        symbol: "SPY".to_string(),
        history: Vec::new(),
    };

    // Ask is < mean - 2.0 * std_dev so should generate a buy signal
    let buy_quote = Quote {
        symbol: "SPY".to_string(),
        bid: 90.0,
        ask: 90.0,
        biddate: Local::now(),
        askdate: Local::now(),
    };
    match strategy.handle(&buy_quote, &symbol_data) {
        Ok(signal) => {
            assert_eq!(signal, Signal::Buy);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Ask is > mean - 2.0 * std_dev so should generate a sell signal
    let buy_quote = Quote {
        symbol: "SPY".to_string(),
        bid: 150.0,
        ask: 150.0,
        biddate: Local::now(),
        askdate: Local::now(),
    };
    match strategy.handle(&buy_quote, &symbol_data) {
        Ok(signal) => {
            assert_eq!(signal, Signal::Sell);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Ask is within mean - 2.0 * std_dev so should generate no signal
    let buy_quote = Quote {
        symbol: "SPY".to_string(),
        bid: 95.0,
        ask: 95.0,
        biddate: Local::now(),
        askdate: Local::now(),
    };
    match strategy.handle(&buy_quote, &symbol_data) {
        Ok(signal) => {
            assert_eq!(signal, Signal::None);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
