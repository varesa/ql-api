use regex::Regex;
use serde::{Deserialize, Serialize};

const SCALE: f64 = (10000.0+1000.0)/2.0;
const OFFSET: f64 = -10000.0;
const VALUE_THRESHOLD: i64 = -10000;
const VALUE_MIN: i64 = -32768;

fn to_position(value: i64) -> u8 {
    if value <= VALUE_THRESHOLD {
        0
    } else {
        10.0_f64.powf((value as f64 - OFFSET) / SCALE).round() as u8
    }
}

fn to_value(position: u8) -> i64 {
    if position <= 1 {
        VALUE_MIN
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
    static ref ON_REGEX: Regex = Regex::new(r#"(OK|NOTIFY) set MIXER:Current/InCh/Fader/On (\d+) (\d+) (\d+) "(.*)""#).unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct Fader {
    position: Option<u8>,
    value: Option<i64>,
    comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
                    position: Some(to_position(value)),
                    value: Some(value),
                    comment: Some(comment),
                })
            };
            return serde_json::to_string(&message).expect("Failed to serialize channel");
        }
        
        if let Some(captures) = ON_REGEX.captures(&raw) {
            let channel = captures[2].parse().expect("Failed to parse channel");
            let value = &captures[4] == "1";
            let message = Json1Message {
                channel,
                state: Some(value),
                fader: None,
            };
            return serde_json::to_string(&message).expect("Failed to serialize channel");
        }

        String::new()
    }

    fn to_ql(&self, json: String) -> String {
        let message: Json1Message = serde_json::from_str(&json).expect("Failed to parse JSON");
        let channel = message.channel;
        if let Some(fader) = message.fader {
            let value = if let Some(value) = fader.value {
                value
            } else if let Some(position) = fader.position {
                to_value(position)
            } else {
                panic!("No value or position specified");
            };
            return format!("set MIXER:Current/InCh/Fader/Level {} 0 {}", channel, value)
        }
        if let Some(state) = message.state {
            let value = if state { "1" } else { "0" };
            return format!("set MIXER:Current/InCh/Fader/On {} 0 {}", channel, value)
        }
        String::new()
    }
}
