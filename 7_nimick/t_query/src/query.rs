extern crate regex;
use regex::Regex;
use t::T;
use self::Query::{From, Enable, Disable, Invalid};

//mod t;

static PROMPT_STRING: &'static str = "===>>>";
static DISAMBIG_START: &'static str = "disambiguate your start: ";
static DISAMBIG_DEST: &'static str = "disambiguate your destination: ";
static DISAMBIG_OP: &'static str = "disambiguate your target: ";
static NO_SUCH_START: &'static str = "no such start: ";
static NO_SUCH_DEST: &'static str = "no such destination: ";
static NO_SUCH_DISABLE: &'static str = "no such station to disable: ";
static NO_SUCH_ENABLE: &'static str = "no such station to enable: ";
static SWITCH: &'static str = "---switch from {} to {}";
static ENSURE: &'static str = "---ensure you are on {}";

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
            return From(cap.at(0).unwrap().trim(),
                        cap.at(1).unwrap().trim());
        },
        None => {}
    }
    match disable_regex.captures(line) {
        Some(cap) => {
            return Disable(cap.at(0).unwrap().trim());
        },
        None => {}
    }
    match enable_regex.captures(line) {
        Some(cap) => {
            return Enable(cap.at(0).unwrap().trim());
        },
        None => {}
    }
    Invalid
}


#[allow(unused_must_use)]
pub fn query_user<W: Writer, R: Buffer>(output: &mut W, input: &mut R,
                                    t: &T) {
    let (from_regex, disable_regex, enable_regex) = compile_regexes();

    // Why doesn't this work?
    /*let parse_line_ = |&: line: String| -> Query {
        parse_line(line.as_slice(), &from_regex, &disable_regex, &enable_regex)
    };*/

    output.write_str(PROMPT_STRING);
    output.flush();
    for line in input.lines() {
        match parse_line(line.unwrap().as_slice(), &from_regex, &disable_regex, &enable_regex) {
            From(from, to) => {
                println!("from query");
            },
            Disable(station) => {
                println!("disable query");
            },
            Enable(station) => {
                println!("enable query");
            },
            Invalid => {
                println!("invalid");
            }
        }
        output.write_str(PROMPT_STRING);
        output.flush();
    }
}

