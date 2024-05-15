use tungstenite::{connect, Message};

fn main() {
    // let (mut socket, response) =
    //     connect("wss://ws.tradier.com/v1/markets/events").expect("Can't connect");
    match connect("wss://ws.tradier.com/v1/markets/events") {
        Ok((mut socket, response)) => {
            // let body = String::from_utf8(response.body().clone().unwrap()).unwrap();
            // println!("Response:, {}", body);

            println!("Connected to the server");
            println!("Response HTTP code: {}", response.status());
            println!("Response contains the following headers:");
            for (ref header, _value) in response.headers() {
                println!("* {}", header);
            }

            socket
                .send(Message::Text(
                    "{\"symbols\": [\"SPY\"], \"sessionid\": \"5f9687cd-bc07-4cae-9f15-42995931eb60\", \"linebreak\": true}".into(),
                ))
                .unwrap();
            loop {
                let msg = socket.read().expect("Error reading message");
                println!("Received: {}", msg);
            }
            // socket.close(None);
        }

        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
