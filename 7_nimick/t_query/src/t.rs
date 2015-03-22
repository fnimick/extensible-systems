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

macro_rules! string_set {
    ($( $x:expr ),* ) => {{
        let mut temp_set = HashSet::new();
        $(
            temp_set.insert(String::from_str($x));
        )*
        temp_set
    }};
}

#[derive(Show, PartialEq)]
enum TQueryResult<'a> {
    TOk(Vec<TStep>),
    DisambiguateStart(Vec<String>),
    DisambiguateDestination(Vec<String>),
    NoSuchStart,
    NoSuchDest,
    NoSuchPath
}

#[derive(Show, PartialEq)]
enum TOperationResult<'a> {
    Successful,
    DisambiguateOp(Vec<String>),
    NoSuchStationOp
}

#[derive(Show, PartialEq)]
enum TStep {
    // Station, line name
    Station(String, String),
    // Station, line name
    Switch(String, String),
    // line name
    Ensure(String)
}

#[derive(Show, PartialEq)]
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
            tee: "mbta".to_string(),
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
            let l = line.trim();
            if l.starts_with("-") || l.is_empty() {
                continue;
            }
            self.stations.insert(l.to_string());
        }
    }

    fn rebuild_graph(&mut self) {
        // TODO rebuild the graph given the current disabled list
    }

    /// Find a path "from" the start "to" the destination
    pub fn find_path(&self, from: &str, to: &str) -> TQueryResult {
        let start = return_some_vec!(self.disambiguate_station(from), DisambiguateStart, NoSuchStart);
        let dest = return_some_vec!(self.disambiguate_station(to), DisambiguateDestination, NoSuchDest);
        // Find the path from the start to the destination, or return NoSuchPath
        NoSuchPath
    }

    /// Modify the given station to set it to be enabled/disabled
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

    /// Enable the given station. This function is a wrapper for modify_station
    pub fn enable_station(&mut self, station: &str) -> TOperationResult {
        self.modify_station(station, true)
    }

    /// Disable the given station. This function is a wrapper for modify_station
    pub fn disable_station(&mut self, station: &str) -> TOperationResult {
        self.modify_station(station, false)
    }

    /// Return a suggested station or list of sorted station suggestions if the
    /// given station is close but not a complete match to an actual station
    /// (or set of actual stations)
    ///
    /// Assumption: 'Close but not a complete match' means that the given
    ///             string is a substring of an actual station
    fn disambiguate_station(&self, station: &str) -> DisambiguationResult {
        if self.stations.contains(station) {
            return DisambiguationResult::Station(station.to_string());
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
            ret_vec.sort();
            DisambiguationResult::Suggestions(ret_vec)
        }
    }

    /// Print to the output writer the result of calling find_path on the T.
    pub fn output_find_path<W: Writer>(&self, path: TQueryResult, from: &str,
                                       to: &str, output: &mut W) {
        match path {
            TOk(steps) => { print_steps(steps, output); },
            DisambiguateStart(suggestions) => { print_vec(DISAMBIG_START, suggestions, output); },
            DisambiguateDestination(suggestions) => { print_vec(DISAMBIG_DEST, suggestions,
                                                                output); },
            NoSuchStart => { print_str(NO_SUCH_START, from, output); },
            NoSuchDest => { print_str(NO_SUCH_DEST, to, output); },
            NoSuchPath => { output.write_str(NO_SUCH_PATH); }
        }
    }

    /// Output the result of calling enable or disable a station
    fn output_toperation_result<W: Writer>(&self, result: TOperationResult,
                                           station: &str, no_such: &str, output: &mut W) {
        match result {
            Successful => { output.write_str(SUCCESS_OP); },
            DisambiguateOp(suggestions) => { print_vec(DISAMBIG_OP, suggestions, output); },
            NoSuchStationOp => { print_str(no_such, station, output); }
        }
    }

    /// Print to the output writer the result of enabling the given station
    pub fn output_enable_station<W: Writer>(&self, station: &str,
                                            enabled: TOperationResult, output: &mut W) {
        self.output_toperation_result(enabled, station, NO_SUCH_ENABLE, output)
    }

    /// Print to the output writer the result of disabling the given station
    pub fn output_disable_station<W: Writer>(&mut self, station: &str,
                                             disabled: TOperationResult, output: &mut W) {
        self.output_toperation_result(disabled, station, NO_SUCH_DISABLE, output)
    }
}

#[cfg(test)]
mod t_tests {
    use super::T;
    use super::{TOperationResult, TQueryResult, DisambiguationResult};
    use super::{NO_SUCH_ENABLE, NO_SUCH_DISABLE, SUCCESS_OP, DISAMBIG_OP, DISAMBIG_START,
                DISAMBIG_DEST, NO_SUCH_START, NO_SUCH_DEST, NO_SUCH_PATH};
    use super::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
    use super::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, NoSuchPath};
    use super::TStep::{Station, Switch, Ensure};
    use std::io::MemWriter;
    use std::collections::HashSet;

    #[test]
    fn test_read_data_file() {
        let mut t = T::new();
        t.read_data_file("data/red.dat");
        let expect = string_set![
            "Alewife Station", "Davis Station", "Porter Square Station",
            "Harvard Square Station", "Central Square Station",
            "Kendall Station", "Charles/MGH Station", "Park Street Station",
            "Downtown Crossing Station", "South Station", "Broadway Station",
            "Andrew Station", "JFK/UMass Station", "North Quincy Station",
            "Wollaston Station", "Quincy Center Station",
            "Quincy Adams Station", "Braintree Station", "Savin Hill Station",
            "Fields Corner Station", "Shawmut Station", "Ashmont Station",
            "Cedar Grove Station", "Butler Station", "Milton Station",
            "Central Avenue Station", "Valley Road Station",
            "Capen Street Station", "Mattapan Station"
        ];
        assert_eq!(expect.len(), t.stations.len());
        for station in t.stations.iter() {
            assert!(expect.contains(station));
        }
    }

    #[test]
    fn test_rebuild_graph() {}

    // TODO: Come back to this
    fn test_find_path() {
        let expect1 = TOk(vec![Station("South Station".to_string(),
                                      "red".to_string()),
                              Station("Broadway Station".to_string(),
                                      "red".to_string()),
                              Station("Andrew Station".to_string(),
                                      "red".to_string())]);
        run_find_path_test("South Station", "Andrew Station", expect1);

        let expect2 = DisambiguateStart(vec!["South Station".to_string(),
                                             "South Street Station".to_string()]);
        run_find_path_test("South", "Andrew Station", expect2);
        run_find_path_test("asdf", "Downtown Crossing Station", NoSuchStart);

        let expect3 = DisambiguateDestination(vec!["Andrew Station".to_string()]);
        run_find_path_test("South Station", "Andrew", expect3);
        run_find_path_test("Downtown Crossing Station", "asdf", NoSuchDest);

        let mut t = T::new();
        t.load();
        t.modify_station("Kendall Station", false);
        assert_eq!(t.find_path("Alewife Station", "Braintree Station"), NoSuchPath);
    }

    fn run_find_path_test(start: &str, end: &str, expect: TQueryResult) {
        let mut t = T::new();
        t.load();
        let result = t.find_path(start, end);
        assert_eq!(result, expect);
    }

    #[test]
    fn test_modify_station() {
        let station = "South Station";
        let mut t = T::new();
        t.load();
        assert!(!t.disabled.contains(station));
        t.modify_station(station, false);
        assert!(t.disabled.contains(station));
        t.modify_station(station, true);
        assert!(!t.disabled.contains(station));
        t.disable_station(station);
        t.disable_station(station);
        assert!(t.disabled.contains(station));
        t.enable_station(station);
        t.enable_station(station);
        assert!(!t.disabled.contains(station));
    }

    #[test]
    fn test_disambiguate_station() {
        let mut t = T::new();
        t.load();
        assert_eq!(t.disambiguate_station("Andrew Station"),
                   DisambiguationResult::Station("Andrew Station".to_string()));
        assert_eq!(t.disambiguate_station("Andrew"),
                   DisambiguationResult::Station("Andrew Station".to_string()));

        let expect = string_set!["Tufts Medical Center Station",
                                 "Quincy Center Station",
                                 "Malden Center Station",
                                 "Government Center Station",
                                 "Hynes Convention Center"];
        let suggestions = match t.disambiguate_station("Center") {
            DisambiguationResult::Suggestions(stations) => stations,
            DisambiguationResult::Station(station) => panic!("Bang!")
        };
        for station in suggestions.iter() {
            assert!(expect.contains(station));
        }
    }

    #[test]
    fn test_output_toperation_result() {
        run_test_output_toperation("Andrew Station", false, SUCCESS_OP);
        run_test_output_toperation("South", false,
                                   format!("{}{}{}{}", DISAMBIG_OP,
                                           "South Station ",
                                           "South Street Station ",
                                           "\n").as_slice());
        run_test_output_toperation("asdf", false,
                                   format!("{}{}", NO_SUCH_DISABLE, "asdf\n").as_slice());
        run_test_output_toperation("asdf", true,
                                   format!("{}{}", NO_SUCH_ENABLE, "asdf\n").as_slice());
    }

    /// Test the output of enabling or disabling a station
    fn run_test_output_toperation(station: &str, enable: bool, expect: &str) {
        let mut w = MemWriter::new();
        let mut t = T::new();
        t.load();

        let result: TOperationResult;
        if enable {
            result = t.enable_station(station);
            t.output_enable_station(station, result, &mut w);
        } else {
            result = t.disable_station(station);
            t.output_disable_station(station, result, &mut w);
        }

        assert_eq!(expect, String::from_utf8(w.into_inner()).unwrap());
    }

    // TODO: Come back to this
    fn test_output_find_path() {
        let mut t = T::new();
        t.load();

        let (from, to) = ("South Station", "Andrew Station");
        let expect = concat!("South Station, take red\n",
                             "Broadway Station, take red\n",
                             "Andrew Station, take red\n");
        run_test_output_find_path(&t, t.find_path(from, to), from, to, expect);
    }

    /// Test the output of finding a path
    fn run_test_output_find_path(t: &T, path: TQueryResult,
                                 from: &str, to: &str, expect: &str) {
        let mut w = MemWriter::new();
        t.output_find_path(path, from, to, &mut w);
        assert_eq!(expect, String::from_utf8(w.into_inner()).unwrap());
    }
}

/// Print steps to the output writer
fn print_steps<W: Writer>(steps: Vec<TStep>, output: &mut W) {
    for step in steps.into_iter() {
        match step {
            Station(station, line) => { write!(output, "{}, take {}\n", station, line); },
            Switch(one, two) => { write!(output, "---switch from {} to {}\n", one, two); },
            Ensure(line) => { write!(output, "---ensure you are on {}\n", line); }
        }
    }
}

#[cfg(test)]
mod print_steps_tests {
    use super::print_steps;
    use super::TStep::{Station, Switch, Ensure};
    use std::io::MemWriter;

    #[test]
    fn test_print_vec() {
        let mut w = MemWriter::new();
        let v = vec![Station("a".to_string(), "b".to_string()),
            Switch("c".to_string(), "d".to_string()), Ensure("e".to_string())];
        print_steps(v, &mut w);
        assert_eq!(w.get_ref(), concat!("a, take b\n",
                                        "---switch from c to d\n",
                                        "---ensure you are on e\n").as_bytes());
    }
}

/// Print the vector of Strings to the writer, preceeded by the header
#[allow(unused_must_use)]
fn print_vec<W: Writer>(header: &str, vec: Vec<String>, output: &mut W) {
    output.write_str(header);
    for station in vec.into_iter() {
        output.write_str(station.as_slice());
        output.write_str(" ");
    }
    output.write_str("\n");
}

#[cfg(test)]
mod print_vec_tests {
    use super::print_vec;
    use std::io::MemWriter;

    #[test]
    fn test_print_vec() {
        let mut w = MemWriter::new();
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        print_vec("header", v, &mut w);
        assert_eq!(w.get_ref(), String::from_str("headera b c \n").as_bytes());
    }
}

/// Print the header and given str to the writer
#[allow(unused_must_use)]
fn print_str<W: Writer>(header: &str, s: &str, output: &mut W) {
    output.write_str(header);
    output.write_str(s);
    output.write_str("\n");
}

#[cfg(test)]
mod print_str_tests {
    use super::print_str;
    use std::io::MemWriter;

    #[test]
    fn test_print_str() {
        let mut w = MemWriter::new();
        print_str("header", "string", &mut w);
        assert_eq!(w.get_ref(), String::from_str("headerstring\n").as_bytes());
    }
}

/// Open the file as given by filename in the form of a Buffered Reader
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}
