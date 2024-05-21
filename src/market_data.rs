use crate::tradier_date_format;
use chrono::{DateTime, Local};
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
    fn subscribe(&mut self) -> Result<Receiver<Quote>, String>;
    fn unsubscribe(&mut self, subscriber: Receiver<Quote>) -> Result<(), String>;
}

pub fn new(access_token: String) -> Box<dyn MarketDataService> {
    Box::new(MarketData {
        access_token,
        socket: None,
        symbols: HashSet::new(),
        subscribers: Arc::new(Mutex::new(Vec::new())),
    })
}

#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    symbol: String,
    bid: f64,
    ask: f64,
    #[serde(with = "tradier_date_format")]
    biddate: DateTime<Local>,
    #[serde(with = "tradier_date_format")]
    askdate: DateTime<Local>,
}

struct MarketData {
    access_token: String,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    symbols: HashSet<String>,
    subscribers: Arc<Mutex<Vec<(Sender<Quote>, Receiver<Quote>)>>>,
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
    // @todo Lose symbols here - unless we simplify things and collect all symbols in main()...
    fn init(&mut self, symbols: Vec<String>) -> Result<JoinHandle<String>, String> {
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
                socket
                    .send(Message::Text(message))
                    .expect("Error sending message");

                let subscribers = self.subscribers.clone();
                let handle: JoinHandle<String> = std::thread::spawn(move || loop {
                    let msg = socket
                        .read()
                        .expect("Error reading message")
                        .into_text()
                        .expect("Error converting message to text");
                    let quote =
                        serde_json::from_str::<Quote>(msg.as_str()).expect("Error parsing JSON");
                    println!("Received Quote: {:?}", quote);
                    for subscriber in subscribers.lock().unwrap().iter() {
                        subscriber.0.send(quote.clone()).unwrap();
                    }
                });
                Ok(handle)
            }

            Err(e) => Err(e.to_string()),
        }
    }

    fn subscribe(&mut self) -> Result<Receiver<Quote>, String> {
        let (sender, receiver) = unbounded();
        let subscriber = receiver.clone();
        self.subscribers
            .lock()
            .and_then(|mut s| Ok(s.push((sender, receiver))))
            .map_err(|e| e.to_string())
            .and_then(|_| Ok(subscriber))
    }

    fn unsubscribe(&mut self, subscriber: Receiver<Quote>) -> Result<(), String> {
        match self.subscribers.lock() {
            Ok(mut guard) => {
                let subscribers: &mut Vec<(Sender<Quote>, Receiver<Quote>)> = &mut *guard;
                if let Some(index) = subscribers
                    .iter()
                    .position(|(_, r)| std::ptr::eq(r, &subscriber))
                {
                    subscribers.remove(index);
                    Ok(())
                } else {
                    return Err("No such subscriber found".to_string());
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
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
