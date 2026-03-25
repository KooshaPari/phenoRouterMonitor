use anyhow::{Context, Result};
use reqwest::{Method, Response};
use serde::{Serialize, de::DeserializeOwned};

pub(super) async fn request_json<T: Serialize + ?Sized>(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    body: &T,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .context("request with json body failed")
}

pub(super) async fn request_json_value(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    body: &serde_json::Value,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .context("request with json body failed")
}

pub(super) async fn request_raw_body(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    raw_body: &str,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .body(raw_body.to_string())
        .send()
        .await
        .context("request with raw body failed")
}

pub(super) async fn request_without_body(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .send()
        .await
        .context("request without body failed")
}

pub(super) async fn read_text_response(response: Response, context: &str) -> Result<String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Plane.so API error {status}: {body}");
    }

    response.text().await.context(context.to_owned())
}

pub(super) async fn read_json_response<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Plane.so API error {status}: {body}");
    }

    response.json().await.context(context.to_owned())
}
