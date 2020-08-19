use csv::{ReaderBuilder, StringRecord, Trim};
use serde::export::Formatter;
use serde::Deserialize;
use serde::Serialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

pub struct YdkRecord {
    pub id: String,
    pub count: i32,
}

pub type YdkRecords = Vec<YdkRecord>;

pub fn get_ydk_records(file_path: &str) -> Result<YdkRecords> {
    let file = File::open(file_path)?;
    let rdr = BufReader::new(file);

    // We preserve ordering by having another vector that stores IDs in its canonical ordering
    let mut map = HashMap::new();
    let mut ids = Vec::new();

    for line in rdr.lines() {
        let line = line?;
        match line.chars().next() {
            None => {}
            Some(c) => {
                if c.is_numeric() {
                    let count = match map.entry(line) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => {
                            // FIXME: can we prevent copying here? Can't store ref to map key.
                            ids.push(v.key().clone());
                            v.insert(0)
                        }
                    };

                    *count = *count + 1;
                }
            }
        }
    }

    Ok(ids
        .into_iter()
        .map(|id| {
            let count = *map.get(id.as_str()).expect("Unreachable");
            YdkRecord { id, count }
        })
        .collect())
}
