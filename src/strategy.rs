enum StrategyType {
    MeanReversion { strategy_params: Vec<String> },
}

impl StrategyType {
    fn make(self, params: Vec<String>) -> Box<dyn Strategy> {
        match self {
            Self::MeanReversion { strategy_params } => {
                Box::new(MeanReversion::new(strategy_params))
            }
        }
    }
}

trait Strategy {
    fn run(&self);
}

struct MeanReversion {
    strategy_params: Vec<String>,
}

impl MeanReversion {
    fn new(strategy_params: Vec<String>) -> Self {
        MeanReversion { strategy_params }
    }
}

impl Strategy for MeanReversion {
    fn run(&self) {
        println!(
            "Running Mean Reversion strategy with params: {:?}",
            self.strategy_params
        );
    }
}
