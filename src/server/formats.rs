use regex::Regex;
use serde::{Deserialize, Serialize};

const SCALE: f64 = (32768.0+1000.0)/2.0;
const OFFSET: f64 = -32768.0;

fn to_position(value: i64) -> u8 {
    10.0_f64.powf((value as f64 - OFFSET) / SCALE).round() as u8
}

fn to_value(position: u8) -> i64 {
    if position == 0 {
        (0.0 + OFFSET) as i64
    } else {
        ((position as f64).log10() * SCALE + OFFSET).round() as i64
    }
}

pub trait TransportFormat: Send + Sync {
    fn from_ql(&self, raw: String) -> String;
    fn to_ql(&self, alternative: String) -> String;
}

pub struct Raw {}

impl TransportFormat for Raw {
    fn from_ql(&self, raw: String) -> String {
        raw
    }

    fn to_ql(&self, raw: String) -> String {
        raw
    }
}

pub struct Json1 {}
lazy_static! {
    static ref FADER_REGEX: Regex = Regex::new(r#"(OK|NOTIFY) set MIXER:Current/InCh/Fader/Level (\d+) (\d+) (-?\d+) "(.*)""#).unwrap();
    //static ref FADER_REGEX: Regex = Regex::new("(.*)").unwrap();
}

#[derive(Serialize, Deserialize)]
struct Fader {
    position: u8,
    value: i64,
    comment: String,
}

#[derive(Serialize, Deserialize)]
struct Json1Message {
    channel: u8,
    fader: Option<Fader>,
    state: Option<bool>
}

impl TransportFormat for Json1 {
    fn from_ql(&self, raw: String) -> String {
        if let Some(captures) = FADER_REGEX.captures(&raw) {
            let channel = captures[2].parse().expect("Failed to parse channel");
            let value = captures[4].parse().expect("Failed to parse value");
            let comment = captures[5].parse().expect("Failed to parse comment");
            let message = Json1Message {
                channel,
                state: None,
                fader: Some(Fader {
                    position: to_position(value),
                    value,
                    comment,
                })
            };
            return serde_json::to_string(&message).expect("Failed to serialize channel");
        }

        String::new()
    }

    fn to_ql(&self, _json: String) -> String {
        String::new()
    }
}