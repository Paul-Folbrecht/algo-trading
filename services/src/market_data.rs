use crossbeam_channel::{Receiver, Sender};
use domain::domain::Quote;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tungstenite::{connect, Message};

pub trait MarketDataService {
    fn init(
        &self,
        shutdown: Arc<AtomicBool>,
        symbols: Vec<String>,
    ) -> Result<JoinHandle<()>, String>;
    fn subscribe(&self) -> Result<Receiver<Quote>, String>;
    fn unsubscribe(&self, subscriber: &Receiver<Quote>) -> Result<(), String>;
}

pub fn new(access_token: String) -> Arc<impl MarketDataService> {
    Arc::new(implementation::MarketData {
        access_token,
        symbols: HashSet::new(),
        subscribers: Arc::new(Mutex::new(Vec::new())),
    })
}

mod implementation {
    use super::*;
    use std::{net::TcpStream, thread, time::Duration};
    use tungstenite::{stream::MaybeTlsStream, WebSocket};

    #[derive(Deserialize)]
    struct AuthResponse {
        stream: Stream,
    }

    #[derive(Deserialize)]
    struct Stream {
        sessionid: String,
    }

    pub struct MarketData {
        pub access_token: String,
        pub symbols: HashSet<String>,
        pub subscribers: Arc<Mutex<Vec<(Sender<Quote>, Receiver<Quote>)>>>,
    }

    impl MarketDataService for MarketData {
        fn init(
            &self,
            shutdown: Arc<AtomicBool>,
            symbols: Vec<String>,
        ) -> Result<JoinHandle<()>, String> {
            let token = self.access_token.clone();
            let subscribers = self.subscribers.clone();

            let handle = thread::spawn(move || {
                while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    match authenticate_and_connect(&token, symbols.clone()) {
                        Ok(mut socket) => {
                            while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                                match socket.read() {
                                    Ok(msg) => {
                                        handle_quote(msg, subscribers.clone());
                                    }
                                    Err(e) => {
                                        println!("MarketDataService: Error reading message - possible EOD/inactivity connection close: {}", e);
                                        println!("MarketDataService: Reconnecting unless service shutdown");
                                        thread::sleep(Duration::from_secs(1));
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("MarketDataService: Error connecting: {}", e);
                            thread::sleep(Duration::from_secs(5));
                        }
                    }
                }

                println!("MarketDataService shutting down");
            });

            Ok(handle)
        }

        fn subscribe(&self) -> Result<Receiver<Quote>, String> {
            let (sender, receiver) = crossbeam_channel::unbounded();
            let subscriber = receiver.clone();
            self.subscribers
                .lock()
                .map(|mut s| s.push((sender, receiver)))
                .map_err(|e| e.to_string())
                .map(|_| subscriber)
        }

        fn unsubscribe(&self, subscriber: &Receiver<Quote>) -> Result<(), String> {
            match self.subscribers.lock() {
                Ok(mut guard) => {
                    let subscribers: &mut Vec<(Sender<Quote>, Receiver<Quote>)> = &mut guard;
                    if let Some(index) = subscribers
                        .iter()
                        .position(|(_, r)| std::ptr::eq(r, subscriber))
                    {
                        subscribers.remove(index);
                        Ok(())
                    } else {
                        Err("No such subscriber found".to_string())
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        }
    }

    fn handle_quote(msg: Message, subscribers: Arc<Mutex<Vec<(Sender<Quote>, Receiver<Quote>)>>>) {
        let msg = msg.into_text().expect("Error converting message to text");
        let quote = serde_json::from_str::<Quote>(msg.as_str()).expect("Error parsing JSON");
        println!("MarketDataService received quote: {:?}", quote);

        for subscriber in subscribers.lock().unwrap().iter() {
            match subscriber.0.send(quote.clone()) {
                Ok(_) => (),
                Err(e) => println!(
                    "MarketDataService: Error sending quote to subscriber: {}",
                    e
                ),
            }
        }
    }

    fn authenticate_and_connect(
        access_token: &str,
        symbols: Vec<String>,
    ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, String> {
        let response = authenticate(access_token).map_err(|e| e.to_string())?;
        let session_id = &response.stream.sessionid;
        let symbols_json = serde_json::to_string(&symbols).expect("Error serializing symbols");

        match connect("wss://ws.tradier.com/v1/markets/events") {
            Ok((mut socket, _)) => {
                let message = format!(
                    "{{\"symbols\": {}, \"sessionid\": \"{}\", \"filter\": [\"quote\"], \"linebreak\": true}}",
                    symbols_json,
                    session_id
                );
                socket
                    .send(Message::Text(message))
                    .expect("Error sending message");

                Ok(socket)
            }

            Err(e) => Err(e.to_string()),
        }
    }

    fn authenticate(access_token: &str) -> reqwest::Result<AuthResponse> {
        reqwest::blocking::Client::new()
            .post("https://api.tradier.com/v1/markets/events/session")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .header(CONTENT_LENGTH, "0")
            .send()?
            .json::<AuthResponse>()
    }
}

#[cfg(test)]
#[path = "./tests/market_data_test.rs"]
mod market_data_test;
