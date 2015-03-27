#[doc="
    Module: Query

    This module handles the user's queries on the MBTA structure. It handles
    all interaction with the user, including parsing.
"]

extern crate regex;

#[cfg(not(test))]
use std::sync::{Arc, Mutex};
#[cfg(not(test))]
use t::T;
#[cfg(not(test))]
use print;
#[cfg(not(test))]
use print::{output_find_path, output_enable_station, output_disable_station};

use regex::Regex;
use self::Query::{From, Enable, Disable, Invalid};

#[cfg(not(test))]
static PROMPT_STRING: &'static str = "===>>> ";
#[cfg(not(test))]
static INVALID_QUERY: &'static str = "Invalid command format.\n";

macro_rules! regex (
    ($s:expr) => (regex::Regex::new($s).unwrap());
    );

#[derive(Show, PartialEq, Eq)]
enum Query<'a> {
    From(&'a str, &'a str),
    Enable(&'a str),
    Disable(&'a str),
    Invalid
}

struct Parser {
    from_regex: regex::Regex,
    disable_regex: regex::Regex,
    enable_regex: regex::Regex
}

impl Parser {

    /// Parse the given user input to return a Query
    fn parse_line<'a>(&self, line: &'a str) -> Query<'a> {
        match self.from_regex.captures(line) {
            Some(cap) => {
                return From(cap.at(1).unwrap().trim(),
                            cap.at(2).unwrap().trim());
            },
            None => {}
        }
        match self.disable_regex.captures(line) {
            Some(cap) => {
                return Disable(cap.at(1).unwrap().trim());
            },
            None => {}
        }
        match self.enable_regex.captures(line) {
            Some(cap) => {
                return Enable(cap.at(1).unwrap().trim());
            },
            None => {}
        }
        Invalid
    }
}

#[cfg(test)]
mod parser_tests {
    use super::compile_regexes;
    use super::Query::{From, Disable, Enable};

    #[test]
    fn test_parse_line() {
        let p = compile_regexes();
        assert_eq!(From("South", "Ruggles"), p.parse_line("from South to Ruggles"));
        assert_eq!(Disable("Ruggles"), p.parse_line("disable Ruggles"));
        assert_eq!(Enable("Ruggles"), p.parse_line("enable Ruggles"));
    }
}

/// Create the parser
fn compile_regexes() -> Parser {
    Parser {
        from_regex: regex!(r"from ([a-zA-Z\. ]+) to ([a-zA-Z\. ]+)"),
        disable_regex: regex!(r"disable ([a-zA-Z\. ]+)"),
        enable_regex: regex!(r"enable ([a-zA-Z\. ]+)")
    }
}

#[allow(unused_must_use)]
#[cfg(not(test))]
/// The interface through which the user interacts with the T structure
/// query_user asks the user for a command/operation, and then
/// executes it and prints the response back to the stream
pub fn query_user<BS: Writer + Buffer>(stream: &mut BS, t: Arc<Mutex<T>>) {
    let parser = compile_regexes();
    let mut mbta = t.lock().unwrap();

    stream.write_str(PROMPT_STRING);
    stream.flush();
    while let Ok(line) = stream.read_line() {
        match parser.parse_line(line.as_slice()) {
            From(from, to) => {
                let path = mbta.find_path(from, to);
                print::output_find_path(path, from, to, stream);
            },
            Disable(station) => {
                let disabled = mbta.disable_station(station);
                print::output_disable_station(station, disabled, stream);
            },
            Enable(station) => {
                let enabled = mbta.enable_station(station);
                print::output_enable_station(station, enabled, stream);
            },
            Invalid => {
                stream.write_str(INVALID_QUERY);
            }
        }
        stream.write_str(PROMPT_STRING);
        stream.flush();
    }
}

