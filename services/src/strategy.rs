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
