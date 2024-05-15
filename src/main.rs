use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
//use reqwest::{Client, Method, Response};
use serde::Deserialize;
use tungstenite::{connect, Message};

#[derive(Deserialize, Debug)]
struct AuthResponse {
    stream: Stream,
}

// {"stream":{"url":"https:\/\/stream.tradier.com\/v1\/markets\/events","sessionid":"d0c9b9fc-371f-4a89-bbb5-a02f88082ab8"}}%
#[derive(Deserialize, Debug)]
struct Stream {
    url: String,
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
        Ok((mut socket, response)) => {
            //            let _body = String::from_utf8(response.into_body().unwrap()).unwrap();
            println!("Response HTTP code: {}", response.status());
            println!("Response contains the following headers:");
            for (ref header, _value) in response.headers() {
                println!("* {}", header);
            }

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
