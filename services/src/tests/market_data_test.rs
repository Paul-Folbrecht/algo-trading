use super::*;

#[test]
fn test_subscribe() {
    let access_token = std::env::var("TRADIER_ACCESS_TOKEN").unwrap();
    let service = new(access_token);
    let symbols = vec!["AAPL".to_string()];
    let shutdown = Arc::new(AtomicBool::new(false));
    let _ = service.init(shutdown.clone(), symbols).unwrap();

    match service.subscribe() {
        Ok(rx) => {
            println!("Subscribed to MarketDataService");
            match rx.recv() {
                Ok(quote) => {
                    println!("Received quote:\n{:?}", quote);
                    std::process::exit(0);
                }
                Err(e) => {
                    panic!("Error on receive!: {}", e);
                }
            }
            // @todo How to make this work?
            //shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        Err(e) => panic!("Failed to subscribe to MarketDataService: {}", e),
    }
}
