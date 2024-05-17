use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use serde::Deserialize;
use tungstenite::{connect, Message};

#[derive(Deserialize)]
struct AuthResponse {
    stream: Stream,
}

#[derive(Deserialize)]
struct Stream {
    sessionid: String,
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let access_token = &args[1];
    let response = authenticate(access_token).unwrap();
    let session_id = &response.stream.sessionid;

    match connect("wss://ws.tradier.com/v1/markets/events") {
        Ok((mut socket, _)) => {
            let message = format!(
                "{{\"symbols\": [\"SPY\"], \"sessionid\": \"{}\", \"filter\": [\"quote\"], \"linebreak\": true}}",
                session_id
            );
            socket.send(Message::Text(message)).unwrap();

            loop {
                let msg = socket.read().expect("Error reading message");
                println!("Received: {}", msg);
            }
            //socket.close(None);
        }

        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
