pub mod lib;

use clap::Clap;
use futures::stream::iter;
use futures::StreamExt;
use lib::api::get_card_prices::price_record;
use lib::{get_record, get_record_from_reader, Records};
use reqwest::Client;

#[derive(Clap, Debug)]
struct Opts {
    #[clap(short)]
    file: Option<String>,
    #[clap(short)]
    out: Option<String>,
    #[clap(long)]
    print_total: bool,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let client = Client::default();
    let records = match opts.file {
        None => get_record_from_reader(std::io::stdin()),
        Some(filename) => get_record(filename.as_str()).unwrap(),
    };

    let records: Records = iter(records)
        .then(|x| price_record(x, &client))
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

    for record in records.iter() {
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
