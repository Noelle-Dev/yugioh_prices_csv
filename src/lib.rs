use csv::{ReaderBuilder, StringRecord, Trim};
use serde::export::Formatter;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub tag: Option<String>,
    pub count: Option<i32>,
    pub rarity: Option<String>,
    pub price: Option<f32>,
}

#[derive(Copy, Clone)]
pub enum ArbitrationStrategy {
    MinValue,
    MaxValue,
}

const HEADERS: [&str; 5] = ["name", "tag", "count", "rarity", "price"];
const REQUIRED_HEADERS: [&str; 1] = ["name"];

#[derive(Debug)]
pub enum Error {
    MissingHeaders,
    IllegalHeader(String),
    MissingHeader(&'static str),
    DuplicateHeader(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingHeaders => write!(f, "Missing Headers!"),
            Error::IllegalHeader(hdr) => write!(f, "Illegal Header: '{}'", hdr),
            Error::MissingHeader(hdr) => write!(f, "Missing Required Header: '{}'", hdr),
            Error::DuplicateHeader(hdr) => write!(f, "Duplicate Header: '{}'", hdr),
        }?;
        Ok(())
    }
}

impl std::error::Error for Error {}

pub type Records = Vec<Record>;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn sanitize_header<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<StringRecord> {
    let headers = reader.headers().or(Err(Error::MissingHeaders))?;
    let headers_map: HashMap<&str, usize> = HEADERS
        .iter()
        .map(|header_str| {
            (
                header_str.clone(),
                headers.iter().filter(|x| x == header_str).count(),
            )
        })
        .collect();

    let header_count = headers_map.iter().fold(0usize, |acc, tup| acc + *tup.1);

    if header_count == 0 {
        return Err(Box::new(Error::MissingHeaders));
    }

    let missing_header = REQUIRED_HEADERS
        .iter()
        .find(|x| *headers_map.get(*x).unwrap() == 0);

    if missing_header.is_some() {
        let hdr = missing_header.unwrap().clone();
        return Err(Box::new(Error::MissingHeader(hdr)));
    }

    let duplicate_header = headers_map.iter().find(|(_, count)| **count > 1);

    if duplicate_header.is_some() {
        let hdr = duplicate_header.unwrap().0.clone();
        return Err(Box::new(Error::DuplicateHeader(hdr)));
    }

    let illegal_header = headers.iter().find(|hdr| !HEADERS.contains(hdr));

    if illegal_header.is_some() {
        let hdr = illegal_header.unwrap().to_owned();
        return Err(Box::new(Error::IllegalHeader(hdr)));
    }

    Ok(headers.clone())
}

pub fn get_records_from_reader<R: std::io::Read>(rdr: R) -> Result<Records> {
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(rdr);
    let _headers = sanitize_header(&mut rdr)?;
    let ret: Records = rdr
        .deserialize()
        .map(|res| res.expect("Malformed Record"))
        .collect();
    Ok(ret)
}

pub fn get_records(file_path: &str) -> Result<Records> {
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    get_records_from_reader(buf_reader)
}

pub mod api {
    pub mod get_card_prices {
        use super::super::{ArbitrationStrategy, Record};
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
            format!("http://yugiohprices.com/api/get_card_prices/{}", card_name)
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
    }
}
