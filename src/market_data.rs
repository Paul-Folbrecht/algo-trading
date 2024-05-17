use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;
use std::net::TcpStream;
use tungstenite::{connect, Message};
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub trait MarketDataService {
    fn init(&mut self, symbols: Vec<String>) -> Result<(), String>;
    fn subscribe(&mut self) -> Result<(), String>;
    fn unsubscribe(&mut self) -> Result<(), String>;
}

struct MarketData {
    access_token: String,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
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
    fn init(&mut self, symbols: Vec<String>) -> Result<(), String> {
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
                self.socket = Some(socket);
                Ok(())

                // #todo Loop must be moved to a thread! Otherwise, it will block the main thread.
                // #todo The loop must be stopped when the last user unsubscribes.
                // loop {
                //     let msg = socket.read().expect("Error reading message");
                //     println!("Received: {}", msg);
                //     // Send to subscribers
                //     // Break the loop if no subscribers? In Rust you let threads terminate rather than stop them...
                // }
                //socket.close(None);
            }

            Err(e) => Err(e.to_string()),
        }
    }

    fn subscribe(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn unsubscribe(&mut self) -> Result<(), String> {
        Ok(())
    }
}

pub fn new(access_token: String) -> Box<dyn MarketDataService> {
    Box::new(MarketData {
        access_token,
        socket: None,
    })
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
