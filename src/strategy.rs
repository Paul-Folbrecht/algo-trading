// enum StrategyType {
//     MeanReversion { strategy_params: Vec<String> },
// }

// impl StrategyType {
//     fn make(name: String, params: Vec<String>) -> Box<dyn Strategy> {
//         match name {
//             Self::MeanReversion { strategy_params } => {
//                 Box::new(MeanReversion::new(strategy_params))
//             }
//         }
//     }
// }

#[derive(Debug)]
pub enum Strategy {
    MeanReversion { symbols: Vec<String> },
}

impl Strategy {
    pub fn new(name: String, symbols: Vec<String>) -> Self {
        match name.as_str() {
            "mean-reversion" => Strategy::MeanReversion { symbols },
            _ => panic!("Unknown strategy: {}", name),
        }
    }
}

// pub trait Strategy {}

// pub trait MeanReversion: Strategy {}

// pub struct MeanReversionStrategy {
//     symbols: Vec<String>,
// }

// impl Strategy for MeanReversionStrategy {}

// pub fn new(name: String, symbols: Vec<String>) -> impl Strategy {
//     MeanReversionStrategy { symbols }
// }

// impl Strategy for MeanReversion {
//     fn run(&self) {
//         println!(
//             "Running Mean Reversion strategy with params: {:?}",
//             self.symbols
//         );
//     }
// }
