use crossbeam_channel::{unbounded, Receiver, Sender};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;
use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tungstenite::{connect, Message};
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub trait MarketDataService {
    fn init(&mut self, symbols: Vec<String>) -> Result<JoinHandle<String>, String>;
    fn subscribe(&mut self) -> Result<Receiver<String>, String>;
    fn unsubscribe(&mut self, subscriber: Receiver<String>) -> Result<(), String>;
}

struct MarketData {
    access_token: String,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    symbols: HashSet<String>,
    subscribers: Arc<Mutex<Vec<(Sender<String>, Receiver<String>)>>>,
}

#[derive(Deserialize)]
struct AuthResponse {
    stream: Stream,
}

#[derive(Deserialize)]
struct Stream {
    sessionid: String,
}

impl MarketData {}

impl MarketDataService for MarketData {
    fn init(&mut self, symbols: Vec<String>) -> Result<JoinHandle<String>, String> {
        println!("self.access_token: {}", self.access_token);
        let response = authenticate(&self.access_token).unwrap();
        let session_id = &response.stream.sessionid;
        let symbols_json = serde_json::to_string(&symbols).unwrap();

        match connect("wss://ws.tradier.com/v1/markets/events") {
            Ok((mut socket, _)) => {
                let message = format!(
                    "{{\"symbols\": {}, \"sessionid\": \"{}\", \"filter\": [\"quote\"], \"linebreak\": true}}",
                    symbols_json,
                    session_id
                );
                socket.send(Message::Text(message)).unwrap();
                // @todo Need to hang on to the socket?
                //self.socket = Some(socket);

                let subscribers = self.subscribers.clone();
                // let handle: ScopedJoinHandle<String> = std::thread::scope(|s| {
                //     s.spawn(move || {
                //         loop {
                //             let msg = socket.read().expect("Error reading message");
                //             println!("Received: {}", msg);
                //             for subscriber in subscribers.lock().unwrap().iter() {
                //                 subscriber.0.send(msg.to_string()).unwrap();
                //             }
                //             // Send to subscribers
                //         }
                //     })
                // });
                let handle: JoinHandle<String> = std::thread::spawn(move || loop {
                    let msg = socket.read().expect("Error reading message");
                    println!("Received: {}", msg);
                    for subscriber in subscribers.lock().unwrap().iter() {
                        subscriber.0.send(msg.to_string()).unwrap();
                    }
                });
                Ok(handle)
                //socket.close(None);
            }

            Err(e) => Err(e.to_string()),
        }
    }

    fn subscribe(&mut self) -> Result<Receiver<String>, String> {
        let (sender, receiver) = unbounded();
        let subscriber = receiver.clone();
        self.subscribers.lock().unwrap().push((sender, receiver));
        // @todo ADD TO SYMBOLS...
        Ok(subscriber)
    }

    fn unsubscribe(&mut self, subscriber: Receiver<String>) -> Result<(), String> {
        match self.subscribers.lock() {
            Ok(mut guard) => {
                let subscribers: &mut Vec<(Sender<String>, Receiver<String>)> = &mut *guard;
                if let Some(index) = subscribers
                    .iter()
                    .position(|(_, r)| std::ptr::eq(r, &subscriber))
                {
                    subscribers.remove(index);
                } else {
                    return Err("No such subscriber found".to_string());
                }
            }
            Err(e) => return Err(e.to_string()),
        }
        self.subscribers.lock().unwrap().remove(0);
        Ok(())
    }
}

pub fn new(access_token: String) -> Box<dyn MarketDataService> {
    Box::new(MarketData {
        access_token,
        socket: None,
        symbols: HashSet::new(),
        subscribers: Arc::new(Mutex::new(Vec::new())),
    })
}

fn authenticate(access_token: &str) -> reqwest::Result<AuthResponse> {
    match reqwest::blocking::Client::new()
        .post("https://api.tradier.com/v1/markets/events/session")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(ACCEPT, "application/json")
        .header(CONTENT_LENGTH, "0")
        .send()?
        .json::<AuthResponse>()
    {
        Ok(r) => Ok(r),
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}
