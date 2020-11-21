pub mod api;
pub mod lib;

use crate::api::card_info::convert_ydk_records;
use api::currency_conversion::get_exchange_rate;
use api::get_card_prices::price_record;
use clap::Clap;
use futures::stream::iter;
use futures::StreamExt;
use lib::{get_records, get_records_from_reader, get_ydk_records, Records};
use reqwest::Client;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;
use url::ParseError;

#[derive(Clap, Debug)]
enum ArbitrationStrategy {
    MinValue,
    MaxValue,
}

impl FromStr for ArbitrationStrategy {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Min" | "MinValue" => Ok(ArbitrationStrategy::MinValue),
            "Max" | "MaxValue" => Ok(ArbitrationStrategy::MaxValue),
            _ => Ok(ArbitrationStrategy::MinValue),
        }
    }
}

impl Into<lib::ArbitrationStrategy> for ArbitrationStrategy {
    fn into(self) -> lib::ArbitrationStrategy {
        match self {
            ArbitrationStrategy::MinValue => lib::ArbitrationStrategy::MinValue,
            ArbitrationStrategy::MaxValue => lib::ArbitrationStrategy::MaxValue,
        }
    }
}

fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

#[derive(Clap, Debug)]
struct Opts {
    #[clap(short, about = "Path to input file, otherwise input comes from stdin")]
    file: Option<String>,
    #[clap(short, about = "Path to output file, otherwise output goes to stdout")]
    out: Option<String>,
    #[clap(long, about = "Prints total value of cards to stdout")]
    print_total: bool,
    #[clap(
        short,
        default_value = "Min",
        about = r#"Arbitration strategy when result is ambiguous. 'Min' or 'MinValue' to pick cheapest option. 'Max' or 'MaxValue' to pick most expensive option."#
    )]
    arbitration_strategy: ArbitrationStrategy,
    #[clap(
        short,
        about = "Currency to use for prices (defaults to USD). List of symbols can be found here: https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html"
    )]
    currency: Option<String>,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let client = Client::default();
    let records = match opts.file {
        None => get_records_from_reader(std::io::stdin()),
        Some(filename) => {
            if get_extension(filename.as_str()) == Some("ydk") {
                let ydk_records = get_ydk_records(filename.as_str());
                match ydk_records {
                    Ok(ydk_records) => Ok(convert_ydk_records(ydk_records, &client).await),
                    Err(err) => Err(err),
                }
            } else {
                get_records(filename.as_str())
            }
        }
    };

    if records.is_err() {
        eprint!("{}", records.unwrap_err());
        return;
    }

    let records = records.unwrap();
    let arb_strategy = opts.arbitration_strategy.into();
    let mut records: Records = iter(records)
        .then(|x| price_record(x, &client, arb_strategy))
        .collect()
        .await;

    let writer: Box<dyn std::io::Write> = match opts.out {
        None => Box::new(std::io::stdout()),
        Some(filename) => Box::new(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(filename)
                .unwrap(),
        ),
    };

    let buf_writer = std::io::BufWriter::new(writer);

    let mut writer = csv::Writer::from_writer(buf_writer);

    let exchange_rate = match opts.currency {
        None => 1.0,
        Some(currency) => get_exchange_rate(currency.as_str(), &client).await,
    };

    for record in records.iter_mut() {
        if record.price.is_some() {
            let price = record.price.as_mut().unwrap();
            *price = *price * exchange_rate
        }
        writer.serialize(record).unwrap();
    }

    if opts.print_total {
        let total_value = records.iter().fold(0f32, |acc, record| {
            let count = record.count.unwrap_or(1) as f32;
            acc + record.price.unwrap() * count
        });
        println!("total value: ${}", total_value)
    }
}
