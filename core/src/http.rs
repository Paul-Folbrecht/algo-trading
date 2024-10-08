use backoff::{retry, Error, ExponentialBackoff};
use log::*;
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_LENGTH},
};
use serde::de::DeserializeOwned;
use std::{fmt::Display, io::Read};

pub fn get<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, String> {
    let op = || {
        Client::new()
            .get(url)
            .headers(headers(token))
            .send()
            .map_err(backoff::Error::transient)
    };

    call(op)
}

pub fn post<T: DeserializeOwned>(url: &str, token: &str, body: String) -> Result<T, String> {
    let op = || {
        let response = Client::new()
            .post(url)
            .headers(headers(token))
            .body(body.clone())
            .send()
            .map_err(backoff::Error::transient)
            .unwrap()
            .text();
        info!("\n\nresponse:\n{:?}", response);

        Client::new()
            .post(url)
            .headers(headers(token))
            .body(body.clone())
            .send()
            .map_err(backoff::Error::transient)
    };

    call(op)
}

fn headers(token: &str) -> HeaderMap<HeaderValue> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
    headers
}

fn backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: std::time::Duration::from_millis(100),
        max_interval: std::time::Duration::from_secs(2),
        max_elapsed_time: Some(std::time::Duration::from_secs(5)),
        ..ExponentialBackoff::default()
    }
}

fn call<T, F, E>(op: F) -> Result<T, String>
where
    T: DeserializeOwned,
    F: FnMut() -> Result<Response, Error<E>>,
    E: Display,
{
    retry(backoff(), op)
        .map_err::<String, _>(|e| e.to_string())
        .and_then(|mut response| {
            let mut buf = String::new();
            response
                .read_to_string(&mut buf)
                .expect("HTTP response not valid UTF-8");
            if buf.contains("error") {
                Err(format!("Received Error Response: {}", buf))
            } else {
                serde_json::from_str(&buf)
                    .map_err(|e| format!("Could not parse response body: {:?}", e))
            }
        })
}
