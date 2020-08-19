use crate::lib::{Record, Records, YdkRecords};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct CardInfoData {
    id: usize,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct CardInfoResponse {
    data: Vec<CardInfoData>,
}

pub fn make_url(ydk_records: &YdkRecords) -> Url {
    let mut base_url = Url::parse("https://db.ygoprodeck.com/api/v7/cardinfo.php").expect("");
    let ids: Vec<&str> = ydk_records
        .iter()
        .map(|record| record.id.as_str())
        .collect();

    let joined: String = ids.join(",");

    base_url
        .query_pairs_mut()
        .append_pair("id", joined.as_str());

    base_url
}

pub async fn call(ydk_records: &YdkRecords, client: &Client) -> CardInfoResponse {
    let url = make_url(&ydk_records);
    client
        .get(url)
        .send()
        .await
        .expect("")
        .json()
        .await
        .expect("")
}

pub async fn convert_ydk_records(ydk_records: YdkRecords, client: &Client) -> Records {
    let resp = call(&ydk_records, client).await;
    let mut records = Records::new();

    for record in ydk_records.iter() {
        let card_info = resp
            .data
            .iter()
            .find(|card_info_data| card_info_data.id.to_string() == record.id);

        match card_info {
            Some(card_info) => records.push(Record {
                name: card_info.name.clone(),
                tag: None,
                count: Some(record.count),
                rarity: None,
                price: None,
            }),
            None => { /* TODO: ERROR HANDLING */ }
        }
    }

    records
}
