use csv::Reader;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub tag: String,
    pub name: String,
    pub count: Option<i32>,
    pub rarity: Option<String>,
    pub price: Option<f32>,
}

pub type Records = Vec<Record>;

pub type Result = std::result::Result<Records, std::io::Error>;

pub fn get_record_from_reader<R: std::io::Read>(rdr: R) -> Records {
    let mut rdr = Reader::from_reader(rdr);
    let ret: Records = rdr
        .deserialize()
        .map(|res| res.expect("Malformed Record"))
        .collect();
    ret
}

pub fn get_record(file_path: &str) -> Result {
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    Ok(get_record_from_reader(buf_reader))
}

pub mod api {
    pub mod get_card_prices {
        use super::super::Record;
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
            data: PriceData,
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
        pub async fn price_record(record: Record, client: &Client) -> Record {
            let resp = call(record.name.as_str(), client).await;
            let card_prices = resp
                .data
                .into_iter()
                .find(|x| {
                    let matching_tag = x.print_tag == record.tag;
                    match &record.rarity {
                        None => matching_tag,
                        Some(rarity) => matching_tag && x.rarity == *rarity,
                    }
                })
                .unwrap();

            Record {
                price: Some(card_prices.price_data.data.prices.average),
                ..record
            }
        }
    }
}
