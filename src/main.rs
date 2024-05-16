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

fn authenticate() -> reqwest::Result<AuthResponse> {
    reqwest::blocking::Client::new()
        .post("https://api.tradier.com/v1/markets/events/session")
        .header(AUTHORIZATION, "Bearer RX0IJTkJdbCoS2L5km6RiClQuK9X")
        .header(ACCEPT, "application/json")
        .header(CONTENT_LENGTH, "0")
        .send()?
        .json::<AuthResponse>()
}

fn main() {
    let response = authenticate().unwrap();
    let session_id = &response.stream.sessionid;

    match connect("wss://ws.tradier.com/v1/markets/events") {
        Ok((mut socket, _)) => {
            let message = format!(
                "{{\"symbols\": [\"SPY\"], \"sessionid\": \"{}\", \"linebreak\": true}}",
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
