use self::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, NoSuchPath};
use self::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use self::TStep::{Station, Switch, Ensure};
use std::fmt::{write, Arguments};

static DISAMBIG_START: &'static str = "disambiguate your start: ";
static DISAMBIG_DEST: &'static str = "disambiguate your destination: ";
static DISAMBIG_OP: &'static str = "disambiguate your target: ";
static SUCCESS_OP: &'static str = "done\n";
static NO_SUCH_START: &'static str = "no such start: ";
static NO_SUCH_DEST: &'static str = "no such destination: ";
static NO_SUCH_DISABLE: &'static str = "no such station to disable: ";
static NO_SUCH_ENABLE: &'static str = "no such station to enable: ";
static NO_SUCH_PATH: &'static str = "No path exists.\n";

macro_rules! return_some {
    ($res:expr, $wrapper:expr) => {
        match $res {
            Some(val) => {return $wrapper(val);},
            None => {},
        }
    }
}

enum TQueryResult {
    TOk(Vec<TStep>),
    DisambiguateStart(Vec<String>),
    DisambiguateDestination(Vec<String>),
    NoSuchStart(String),
    NoSuchDest(String),
    NoSuchPath
}

enum TOperationResult {
    Successful,
    DisambiguateOp(Vec<String>),
    NoSuchStationOp
}

enum TStep {
    Station(String, String),
    Switch(String, String),
    Ensure(String)
}

pub struct T {
    tee: String
}

impl T {
    pub fn new() -> T {
        T { tee: "test".to_string() }
    }

    fn find_path(&self, from: &str, to: &str) -> TQueryResult {
        return_some!(self.disambiguate_station(from), DisambiguateStart);
        return_some!(self.disambiguate_station(to), DisambiguateDestination);
        NoSuchPath
    }

    fn enable_station(&mut self, station: &str) -> TOperationResult {
        return_some!(self.disambiguate_station(station), DisambiguateOp);
        Successful
    }

    fn disable_station(&mut self, station: &str) -> TOperationResult {
        return_some!(self.disambiguate_station(station), DisambiguateOp);
        Successful
    }

    fn disambiguate_station(&self, station: &str) -> Option<Vec<String>> {
        None
    }

    pub fn output_find_path<W: Writer>(&self, from: &str, to: &str, output: &mut W) {
        match self.find_path(from, to) {
            TOk(steps) => { print_steps(steps, output); },
            DisambiguateStart(suggestions) => { print_vec(DISAMBIG_START, suggestions, output); },
            DisambiguateDestination(suggestions) => { print_vec(DISAMBIG_DEST, suggestions,
                                                                output); },
            NoSuchStart(s) => { print_str(NO_SUCH_START, s.as_slice(), output); },
            NoSuchDest(s) => { print_str(NO_SUCH_DEST, s.as_slice(), output); },
            NoSuchPath => { output.write_str(NO_SUCH_PATH); }
        }
    }

    pub fn output_enable_station<W: Writer>(&mut self, station: &str, output: &mut W) {
        match self.enable_station(station) {
            Successful => { output.write_str(SUCCESS_OP); },
            DisambiguateOp(suggestions) => { print_vec(DISAMBIG_OP, suggestions, output); },
            NoSuchStationOp => { print_str(NO_SUCH_ENABLE, station, output); }
        }
    }

    pub fn output_disable_station<W: Writer>(&mut self, station: &str, output: &mut W) {
        match self.disable_station(station) {
            Successful => { output.write_str(SUCCESS_OP); },
            DisambiguateOp(suggestions) => { print_vec(DISAMBIG_OP, suggestions, output); },
            NoSuchStationOp => { print_str(NO_SUCH_DISABLE, station, output); }
        }
    }
}

fn print_steps<W: Writer>(steps: Vec<TStep>, output: &mut W) {
    for step in steps.into_iter() {
        match step {
            Station(station, line) => { write!(output, "{}, take {}\n", station, line); },
            Switch(one, two) => { write!(output, "---switch from {} to {}\n", one, two); },
            Ensure(line) => { write!(output, "---ensure you are on {}\n", line); }
        }
    }
}

fn print_vec<W: Writer>(header: &str, vec: Vec<String>, output: &mut W) {
    output.write_str(header);
    for station in vec.into_iter() {
        output.write_str(station.as_slice());
    }
    output.write_str("\n");
}

fn print_str<W: Writer>(header: &str, s: &str, output: &mut W) {
    output.write_str(header);
    output.write_str(s.as_slice());
    output.write_str("\n");
}
