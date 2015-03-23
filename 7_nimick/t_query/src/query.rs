extern crate regex;

use regex::Regex;
use std::sync::{Arc, Mutex};
use t::T;
use self::Query::{From, Enable, Disable, Invalid};

static PROMPT_STRING: &'static str = "===>>> ";
static INVALID_QUERY: &'static str = "Invalid command format.\n";

macro_rules! regex (
    ($s:expr) => (regex::Regex::new($s).unwrap());
    );

enum Query<'a> {
    From(&'a str, &'a str),
    Enable(&'a str),
    Disable(&'a str),
    Invalid
}

fn compile_regexes() -> (Regex, Regex, Regex) {
    (regex!(r"from ([a-zA-Z ]+) to ([a-zA-Z ]+)"),
     regex!(r"disable ([a-zA-Z ]+)"),
     regex!(r"enable ([a-zA-Z ]+)"))
}

fn parse_line<'a>(line: &'a str, from_regex: &Regex, disable_regex: &Regex, enable_regex: &Regex) -> Query<'a> {
    match from_regex.captures(line) {
        Some(cap) => {
            return From(cap.at(1).unwrap().trim(),
                        cap.at(2).unwrap().trim());
        },
        None => {}
    }
    match disable_regex.captures(line) {
        Some(cap) => {
            return Disable(cap.at(1).unwrap().trim());
        },
        None => {}
    }
    match enable_regex.captures(line) {
        Some(cap) => {
            return Enable(cap.at(1).unwrap().trim());
        },
        None => {}
    }
    Invalid
}


#[allow(unused_must_use)]
pub fn query_user<BS: Writer + Buffer>(stream: &mut BS, t: Arc<Mutex<T>>) {
    let (from_regex, disable_regex, enable_regex) = compile_regexes();

    // How can we make this work?
    /*let parse_line_ = |&: line: String| -> Query {
        parse_line(line.as_slice(), &from_regex, &disable_regex, &enable_regex)
    };*/

    let mut mbta = t.lock().unwrap();

    stream.write_str(PROMPT_STRING);
    stream.flush();
    while let Ok(line) = stream.read_line() {
        match parse_line(line.as_slice(), &from_regex, &disable_regex, &enable_regex) {
            From(from, to) => {
                let path = mbta.find_path(from, to);
                mbta.output_find_path(path, from, to, stream);
            },
            Disable(station) => {
                let disabled = mbta.disable_station(station);
                mbta.output_disable_station(station, disabled, stream);
            },
            Enable(station) => {
                let enabled = mbta.enable_station(station);
                mbta.output_enable_station(station, enabled, stream);
            },
            Invalid => {
                stream.write_str(INVALID_QUERY);
            }
        }
        stream.write_str(PROMPT_STRING);
        stream.flush();
    }
}

