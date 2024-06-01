use domain::domain::Quote;

#[derive(Debug)]
pub enum Strategy<'market_data> {
    MeanReversion { symbols: &'market_data Vec<String> },
}

pub trait StrategyHandler {
    fn handle(&self, quote: Quote);
}

impl<'market_data> Strategy<'market_data> {
    pub fn new(name: String, symbols: &'market_data Vec<String>) -> Self {
        match name.as_str() {
            "mean-reversion" => Strategy::MeanReversion { symbols },
            _ => panic!("Unknown strategy: {}", name),
        }
    }
}

impl<'market_data> StrategyHandler for Strategy<'market_data> {
    fn handle(&self, quote: Quote) {
        match self {
            Strategy::MeanReversion { symbols } => {
                println!("MeanReversion strategy handling quote: {:?}", quote);
            }
        }
    }
}
