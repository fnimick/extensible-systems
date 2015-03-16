use self::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, NoSuchPath};
use self::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use self::TStep::{Station, Switch, Ensure};
use std::fmt::{write, Arguments};
use std::collections::HashSet;
use std::io::BufferedReader;
use std::io::fs::File;

static DISAMBIG_START: &'static str = "disambiguate your start: ";
static DISAMBIG_DEST: &'static str = "disambiguate your destination: ";
static DISAMBIG_OP: &'static str = "disambiguate your target: ";
static SUCCESS_OP: &'static str = "done\n";
static NO_SUCH_START: &'static str = "no such start: ";
static NO_SUCH_DEST: &'static str = "no such destination: ";
static NO_SUCH_DISABLE: &'static str = "no such station to disable: ";
static NO_SUCH_ENABLE: &'static str = "no such station to enable: ";
static NO_SUCH_PATH: &'static str = "No path exists.\n";

macro_rules! return_some_vec {
    ($res:expr, $wrapper:expr, $empty:expr) => {
        match $res {
            DisambiguationResult::Suggestions(val) => {
                if val.is_empty() {
                    return $empty;
                }
                return $wrapper(val);
            },
            DisambiguationResult::Station(val) => { val },
        }
    }
}

enum TQueryResult<'a> {
    TOk(Vec<TStep>),
    DisambiguateStart(Vec<String>),
    DisambiguateDestination(Vec<String>),
    NoSuchStart,
    NoSuchDest,
    NoSuchPath
}

enum TOperationResult<'a> {
    Successful,
    DisambiguateOp(Vec<String>),
    NoSuchStationOp
}

enum TStep {
    Station(String, String),
    Switch(String, String),
    Ensure(String)
}

enum DisambiguationResult {
    Station(String),
    Suggestions(Vec<String>)
}

pub struct T<'a> {
    tee: String,
    stations: HashSet<String>,
    disabled: HashSet<String>
}

impl<'a> T<'a> {
    pub fn new() -> T<'a> {
        T {
            tee: "test".to_string(),
            stations: HashSet::new(),
            disabled: HashSet::new()
        }
    }

    pub fn load(&mut self) {
        self.read_data_file("data/blue.dat");
        self.read_data_file("data/green.dat");
        self.read_data_file("data/red.dat");
        self.read_data_file("data/orange.dat");
    }

    fn read_data_file(&mut self, path: &str) {
        let mut reader = open_file(path);
        while let Some(line) = reader.read_line().ok() {
            if line.starts_with("-") {
                continue;
            }
            self.stations.insert(line.trim().to_string());
        }
    }

    fn rebuild_graph(&mut self) {
        // TODO rebuild the graph given the current disabled list
    }

    fn find_path(&self, from: &str, to: &str) -> TQueryResult {
        let start = return_some_vec!(self.disambiguate_station(from), DisambiguateStart, NoSuchStart);
        let dest = return_some_vec!(self.disambiguate_station(to), DisambiguateDestination, NoSuchDest);
        NoSuchPath
    }

    fn modify_station(&mut self, station: &str, enable: bool) -> TOperationResult {
        let station_string = return_some_vec!(self.disambiguate_station(station), DisambiguateOp, NoSuchStationOp);
        println!("Found station: {}", station_string);
        if enable ^ self.disabled.contains(&station_string) {
            return Successful;
        }
        if enable {
            self.disabled.remove(&station_string);
        } else {
            self.disabled.insert(station_string);
        }
        self.rebuild_graph();
        Successful
    }

    fn enable_station(&mut self, station: &str) -> TOperationResult {
        self.modify_station(station, true)
        /*
        return_some_vec!(self.disambiguate_station(station), DisambiguateOp, NoSuchStationOp);
        let station_string = station.to_string();
        if !self.disabled.contains(&station_string) {
            return Successful;
        }
        self.disabled.remove(&station_string);
        self.rebuild_graph();
        Successful*/
    }

    fn disable_station(&mut self, station: &str) -> TOperationResult {
        self.modify_station(station, false)
        /*
        return_some_vec!(self.disambiguate_station(station), DisambiguateOp, NoSuchStationOp);
        let station_string = station.to_string();
        if self.disabled.contains(&station_string) {
            return Successful;
        }
        self.disabled.insert(station_string);
        self.rebuild_graph();
        Successful*/
    }

    fn disambiguate_station(&self, station: &str) -> DisambiguationResult {
        let station_string = station.to_string();
        if self.stations.contains(&station_string) {
            return DisambiguationResult::Station(station_string.clone());
        }
        let mut ret_vec = Vec::new();
        for maybe_match in self.stations.iter() {
            if maybe_match.contains(station) {
                ret_vec.push(maybe_match.clone());
            }
        }
        if ret_vec.len() == 1 {
            DisambiguationResult::Station(ret_vec.pop().unwrap())
        } else {
            DisambiguationResult::Suggestions(ret_vec)
        }
    }

    pub fn output_find_path<W: Writer>(&self, from: &str, to: &str, output: &mut W) {
        match self.find_path(from, to) {
            TOk(steps) => { print_steps(steps, output); },
            DisambiguateStart(suggestions) => { print_vec(DISAMBIG_START, suggestions, output); },
            DisambiguateDestination(suggestions) => { print_vec(DISAMBIG_DEST, suggestions,
                                                                output); },
            NoSuchStart => { print_str(NO_SUCH_START, from, output); },
            NoSuchDest => { print_str(NO_SUCH_DEST, to, output); },
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
        output.write_str(" ");
    }
    output.write_str("\n");
}

fn print_str<W: Writer>(header: &str, s: &str, output: &mut W) {
    output.write_str(header);
    output.write_str(s.as_slice());
    output.write_str("\n");
}

/// Open the file as given by filename in the form of a Buffered Reader
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}
