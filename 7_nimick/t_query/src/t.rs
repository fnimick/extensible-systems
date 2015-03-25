use self::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, NoSuchPath};
use self::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use self::TStep::{Station, Switch, Ensure};
use std::fmt::{write, Arguments};
use std::collections::{HashSet, HashMap};
use std::io::BufferedReader;
use std::io::fs::File;
use graph::{Node, LabeledGraph};

static DISAMBIG_START: &'static str = "disambiguate your start: ";
static DISAMBIG_DEST: &'static str = "disambiguate your destination: ";
static DISAMBIG_OP: &'static str = "disambiguate your target: ";
static SUCCESS_OP: &'static str = "done\n";
static NO_SUCH_START: &'static str = "no such start: ";
static NO_SUCH_DEST: &'static str = "no such destination: ";
static NO_SUCH_DISABLE: &'static str = "no such station to disable: ";
static NO_SUCH_ENABLE: &'static str = "no such station to enable: ";
static NO_SUCH_PATH: &'static str = "No path exists.\n";
static EXCESSIVE_DISABLING_MESSAGE: &'static str = "You've disabled too many things, aborting!";
// how many stations is a transfer equivalent in cost to?
static TRANSFER_COST: Option<usize> = Some(2);

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

#[derive(Show)]
pub struct T<'a> {
    graph: LabeledGraph,
    source_data: HashMap<String, Vec<String>>, // line -> list of stations
    connections: HashSet<(String, String, Option<String>)>,
    stations: HashMap<String, Vec<Node>>, // station name -> list of station nodes
    disabled: HashSet<String>
}

impl<'a> T<'a> {
    pub fn new() -> T<'a> {
        T {
            graph: LabeledGraph::new(),
            source_data: HashMap::new(),
            connections: HashSet::new(),
            stations: HashMap::new(),
            disabled: HashSet::new()
        }
    }

    pub fn load(&mut self) {
        self.read_data_file("data/blue.dat");
        self.read_data_file("data/green.dat");
        self.read_data_file("data/red.dat");
        self.read_data_file("data/orange.dat");
        self.read_connections("data/connections.dat");
        self.rebuild_graph();
    }

    fn read_data_file(&mut self, path: &str) {
        let mut reader = open_file(path);
        let mut rail_line = String::new();
        while let Some(line) = reader.read_line().ok() {
            if line.starts_with("-") {
                rail_line = line.trim_left_matches('-').trim().to_string();
                self.source_data.insert(rail_line.clone(), Vec::new());
                continue;
            }
            let station_name = line.trim().to_string();
            self.source_data.get_mut(&rail_line).unwrap().push(station_name);
        }
    }

    fn read_connections(&mut self, path: &str) {
        let mut reader = open_file(path);
        while let Some(line) = reader.read_line().ok() {
            let mut line_split = line.split(',');
            let one = line_split.next().unwrap().trim().to_string();
            let two = line_split.next().unwrap().trim().to_string();
            let three = match line_split.next() {
                Some(s) => Some(s.trim().to_string()),
                None => None
            };
            self.connections.insert((one, two, three));
        }
    }

    // Rebuilds the graph from source data, taking into account
    // the current disabled station list
    fn rebuild_graph(&mut self) {
        self.stations = HashMap::new();
        self.graph = LabeledGraph::new();
        for (rail_line, station_vec) in self.source_data.iter() {
            let mut prev_node: Option<Node> = None;
            for station_name in station_vec.iter() {
                if self.disabled.contains(station_name) {
                    continue;
                }
                let this_node = Node {
                    station: station_name.clone(),
                    line: rail_line.clone()
                };
                if !self.stations.contains_key(station_name) {
                    self.stations.insert(station_name.clone(), Vec::new());
                }
                let mut node_vec = self.stations.get_mut(station_name).unwrap();
                for existing_node in node_vec.iter() {
                    self.graph.add_edge(existing_node, &this_node, TRANSFER_COST);
                }
                node_vec.push(this_node.clone());
                match prev_node {
                    Some(n) => {
                        self.graph.add_edge(&n, &this_node, None);
                    },
                    None => {}
                };
                prev_node = Some(this_node)
            }
        }
        for &(ref line_one_name, ref line_two_name, ref fallback) in self.connections.iter() {
            let line_one = self.source_data.get(line_one_name).unwrap();
            let station_one = match line_one.iter().filter(|&: station| {
                !self.disabled.contains(*station)
            }).next() {
                Some(s) => s,
                None => { return; }
            };
            let line_two = self.source_data.get(line_two_name).unwrap();
            let station_two = match line_two.iter().rev().filter(|&: station| {
                !self.disabled.contains(*station)
            }).next() {
                Some(s) => s,
                None => {
                    let fallback_line = self.source_data.get(&fallback.clone().unwrap()).unwrap();
                    match fallback_line.iter().rev().filter(|&: station| {
                        !self.disabled.contains(*station)
                    }).next() {
                        Some(s) => s,
                        None => { return; }
                    }
                }
            };
            let node_vec_one = self.stations.get(station_one).unwrap();
            let node_vec_two = self.stations.get(station_two).unwrap();
            assert!(!node_vec_one.is_empty());
            assert!(!node_vec_two.is_empty());
            for node_one in node_vec_one.iter() {
                for node_two in node_vec_two.iter() {
                    self.graph.add_edge(node_one, node_two, TRANSFER_COST);
                }
            }
        }
    }

    /// Find a path "from" the start "to" the destination
    pub fn find_path(&self, start: &str, dest: &str) -> TQueryResult {
        let start = return_some_vec!(self.disambiguate_station(start), DisambiguateStart, NoSuchStart);
        let dest = return_some_vec!(self.disambiguate_station(dest), DisambiguateDestination, NoSuchDest);
        let ref start_node = self.stations.get(&start).unwrap()[0];
        let ref dest_node = self.stations.get(&dest).unwrap()[0];
        match self.graph.find_shortest_path(start_node, dest_node) {
            Some(path) => {
                TOk(interpret_path(path))
            },
            None => NoSuchPath
        }
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
        let station_string = station.to_string();
        let mut ret_vec = Vec::new();
        for maybe_match in self.stations.keys().chain(self.disabled.iter()) {
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

// invariant: path.len() must be > 0
fn interpret_path(path: Vec<Node>) -> Vec<TStep> {
    if path.len() == 1 {
        return Vec::new();
    }

    let mut path_iter = path.into_iter();
    let mut result_vec = Vec::new();
    let mut first_node = path_iter.next().unwrap();
    let mut prev_node = path_iter.next().unwrap();
    process_first_nodes(&mut result_vec, first_node, prev_node.clone());
    for node in path_iter {
        process_nodes(&mut result_vec, prev_node, node.clone());
        prev_node = node;
    }
    prune_end(&mut result_vec);
    result_vec
}

// returns TSteps associated with a transition between two given nodes
fn process_nodes(steps: &mut Vec<TStep>, prev_node: Node, node: Node) {
    if prev_node.line != node.line && prev_node.station != node.station {
        steps.push(Ensure(node.line.clone()));
        steps.push(Station(node.station, node.line));
    } else if prev_node.line != node.line {
        steps.push(Switch(prev_node.line, node.line));
    } else {
        steps.push(Station(node.station, node.line));
    }
}

fn process_first_nodes(steps: &mut Vec<TStep>, prev_node: Node, node: Node) {
    if prev_node.station == node.station {
        steps.push(Station(node.station, node.line));
        return;
    }
    steps.push(Station(prev_node.station.clone(), prev_node.line.clone()));
    process_nodes(steps, prev_node, node);
}

fn prune_end(steps: &mut Vec<TStep>) {
    match steps.pop().unwrap() {
        Station(station, line) => { steps.push(Station(station, line)); },
        _ => {}
    };
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
