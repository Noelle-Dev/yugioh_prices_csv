use reqwest::Client;
use serde::Deserialize;
use std::collections::hash_map::HashMap;
use url::Url;

pub type Rates = HashMap<String, f32>;

#[derive(Debug, Deserialize)]
pub struct ExchangeRatesResponse {
    rates: Rates,
    base: String,
    date: String,
}

const DEFAULT_BASE: &str = "USD";

pub fn make_url(currency: &str) -> Url {
    let mut base_url = Url::parse("https://api.exchangeratesapi.io/latest").expect("");

    base_url
        .query_pairs_mut()
        .append_pair("base", DEFAULT_BASE)
        .append_pair("symbols", currency);

    base_url
}

pub async fn call(currency: &str, client: &Client) -> ExchangeRatesResponse {
    let url = make_url(&currency);
    client
        .get(url)
        .send()
        .await
        .expect("")
        .json()
        .await
        .expect("")
}

pub async fn get_exchange_rate(currency: &str, client: &Client) -> f32 {
    let resp = call(&currency, client).await;
    *resp.rates.get(currency).expect("Exchange Rate Not Found")
}
