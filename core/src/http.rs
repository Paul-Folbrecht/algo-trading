use backoff::{retry, Error, ExponentialBackoff};
use reqwest::{
    blocking::Response,
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_LENGTH},
};
use serde::de::DeserializeOwned;

pub fn get<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, String> {
    let op = || {
        let r = reqwest::blocking::Client::new()
            .get(url)
            .headers(headers(token))
            .send()
            .map_err(backoff::Error::transient);
        println!("\n\n\nresponse:\n{:?}", r.unwrap().text());

        reqwest::blocking::Client::new()
            .get(url)
            .headers(headers(token))
            .send()
            .map_err(backoff::Error::transient)
    };

    call(op)
}

pub fn post<T: DeserializeOwned>(url: &str, token: &str, body: String) -> Result<T, String> {
    let op = || {
        reqwest::blocking::Client::new()
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
    E: std::fmt::Display,
{
    retry(backoff(), op)
        .map_err::<String, _>(|e| e.to_string())
        .and_then(|r| {
            r.json::<T>()
                .map_err::<String, _>(|e| format!("Could not parse response body: {}", e))
        })
}
