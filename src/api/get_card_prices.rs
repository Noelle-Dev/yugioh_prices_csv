use crate::lib::{ArbitrationStrategy, Record};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Prices {
    high: f32,
    low: f32,
    average: f32,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PriceData {
    prices: Prices,
}

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    status: String,
    data: Option<PriceData>,
}

#[derive(Debug, Deserialize)]
pub struct CardPrices {
    name: String,
    print_tag: String,
    rarity: String,
    price_data: PriceResponse,
}

#[derive(Debug, Deserialize)]
pub struct CardPriceResponse {
    status: String,
    data: Vec<CardPrices>,
}

/// Creates the `get_card_prices` URL String given a card name
///
/// # Note
/// We do naive formatting because Url::join(&self, input: &str) behaves incorrectly
/// if input contains special character (for example if input is 'I:P Masquerena'
///
/// # Example
/// ```
/// let url = yugioh_prices_csv::api::get_card_prices::make_url_string("I:P Masquerena");
/// assert_eq!(url, "http://yugiohprices.com/api/get_card_prices/I:P Masquerena")
/// ```
pub fn make_url_string(card_name: &str) -> String {
    format!(
        "http://yugiohprices.com/api/get_card_prices/{}",
        percent_encode(card_name.as_bytes(), NON_ALPHANUMERIC)
    )
}

/// Calls the `get_card_prices` API and return the response
///
/// # Example
///
/// ```rust
/// # async fn doc() {
/// let client = reqwest::Client::default();
/// yugioh_prices_csv::api::get_card_prices::call("I:P Masquerena", &client).await;
/// # }
/// ```
/// FIXME: Return Result<CardPriceResponse, _> instead
pub async fn call(card_name: &str, client: &Client) -> CardPriceResponse {
    let card_price_url: Url = Url::parse(make_url_string(card_name).as_str()).expect("");

    client
        .get(card_price_url)
        .send()
        .await
        .expect("")
        .json()
        .await
        .expect("")
}

/// Price a record by calling the API and finding the price data with matching tag and rarity
///
/// # Example
/// ```
/// use yugioh_prices_csv::Record;
/// use yugioh_prices_csv::api::get_card_prices::price_record;
///
/// let record = Record{
///     tag: "CHIM-EN049".into(),
///     name: "I:P Masquerena".into(),
///     rarity: Some("Ultra Rare".into()),
///     price: None
/// };
///
/// let client = reqwest::Client::default();
/// let record = price_record(record, &client).await;
/// assert!(record.price.is_some());
/// ```
pub async fn price_record(
    record: Record,
    client: &Client,
    arb_strategy: ArbitrationStrategy,
) -> Record {
    let resp = call(record.name.as_str(), client).await;

    let matches = resp.data.into_iter().filter(|card_prices| {
        let matching_tag = match record.tag.as_ref() {
            Some(tag) => *tag == card_prices.print_tag,
            None => true,
        };
        let matching_rarity = match record.rarity.as_ref() {
            Some(rarity) => *rarity == card_prices.rarity,
            None => true,
        };

        card_prices.price_data.data.is_some() && matching_tag && matching_rarity
    });

    let key = |card_prices: &CardPrices| {
        card_prices
            .price_data
            .data
            .as_ref()
            .map_or(0f32, |x| x.prices.average)
    };

    let cmp = |a: &CardPrices, b: &CardPrices| key(a).partial_cmp(&key(b)).unwrap();

    let best_match = match arb_strategy {
        ArbitrationStrategy::MinValue => matches.min_by(cmp),
        ArbitrationStrategy::MaxValue => matches.max_by(cmp),
    };

    match best_match {
        None => record,
        Some(card_prices) => Record {
            price: Some(card_prices.price_data.data.unwrap().prices.average),
            tag: Some(card_prices.print_tag),
            rarity: Some(card_prices.rarity),
            ..record
        },
    }
}
