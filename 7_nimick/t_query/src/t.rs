#[doc="
    Module: T

    This module contains the code specifically related to the MBTA, which is a
    very specialized graph. It exposes operations on the T structure that allow
    an external user to modify the T (enable/disable stations), as well as use
    the T structure to find paths between two stations in the system.
"]

use self::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination,
    NoSuchStart, NoSuchDest, DisabledStart, DisabledDest, NoSuchPath};
use self::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use self::TStep::{Station, Switch, Ensure};
use std::collections::{HashSet, HashMap};
use std::io::BufferedReader;
use std::io::fs::File;
use graph::{Node, LabeledGraph};

// how many stations is a transfer equivalent in cost to?
static TRANSFER_COST: Option<usize> = Some(2);
static NO_COST: Option<usize> = Some(0);
static START_NODE_LABEL: &'static str = "start_node";
static END_NODE_LABEL: &'static str = "end_node";

////////////////////////////////////////////////////////////////////////////
//                              Macros                                    //
////////////////////////////////////////////////////////////////////////////
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

////////////////////////////////////////////////////////////////////////////
//                               Enums                                    //
////////////////////////////////////////////////////////////////////////////
#[derive(Show, PartialEq)]
pub enum TQueryResult<'a> {
    TOk(Vec<TStep>),
    DisambiguateStart(Vec<String>),
    DisambiguateDestination(Vec<String>),
    NoSuchStart,
    NoSuchDest,
    DisabledStart(String),
    DisabledDest(String),
    NoSuchPath
}

#[derive(Show, PartialEq)]
pub enum TOperationResult<'a> {
    Successful,
    DisambiguateOp(Vec<String>),
    NoSuchStationOp
}

#[derive(Show, PartialEq)]
pub enum TStep {
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

////////////////////////////////////////////////////////////////////////////
//                              Structs                                   //
////////////////////////////////////////////////////////////////////////////

#[derive(Show)]
pub struct T<'a> {
    graph: LabeledGraph,

    // Used to reconstruct the T when stations are disabled/enabled
    // line -> list of stations
    source_data: HashMap<String, Vec<String>>,

    // set of connections between lines used to reconstruct the graph
    connections: HashSet<(String, String, Option<String>)>,

    // station name -> list of station nodes that represent the station
    // Stations have 1 or more nodes depending on how many lines connect
    // at the station
    // If a station has more than one node, the second to last node is
    // the 'entrance' node used for unbiased starts of queries, and the
    // last node is the 'exit' node used for unbiased destinations.
    stations: HashMap<String, Vec<Node>>,

    // Set of disabled stations
    disabled: HashSet<String>
}

////////////////////////////////////////////////////////////////////////////
//                              Methods                                   //
////////////////////////////////////////////////////////////////////////////

impl<'a> T<'a> {
    /// Create a new T instance
    pub fn new() -> T<'a> {
        T {
            graph: LabeledGraph::new(),
            source_data: HashMap::new(),
            connections: HashSet::new(),
            stations: HashMap::new(),
            disabled: HashSet::new()
        }
    }

    /// Load the T information from the data files
    pub fn load(&mut self) {
        self.read_data_file("data/blue.dat");
        self.read_data_file("data/green.dat");
        self.read_data_file("data/red.dat");
        self.read_data_file("data/orange.dat");
        self.read_connections("data/connections.dat");
        self.rebuild_graph();
    }

    /// Load a specific data file into the T
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
            if !station_name.is_empty() {
                self.source_data.get_mut(&rail_line).unwrap().push(station_name);
            }
        }
    }

    /// Load a connections file into the T
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

    /// Rebuilds the graph from source data, taking into account
    /// the current disabled station list
    fn rebuild_graph(&mut self) {
        self.stations = HashMap::new();
        self.graph = LabeledGraph::new();
        self.rebuild_lines();
        self.rebuild_connections();
        self.add_unbiased_nodes();
    }

    /// Reconstruct the lines of the T (red, blue, green, orange)
    /// Helper function for rebuild_graph
    fn rebuild_lines(&mut self) {
        for (rail_line, station_vec) in self.source_data.iter() {
            let mut prev_node: Option<Node> = None;
            for station_name in station_vec.iter() {
                // Don't add disabled stations
                if self.disabled.contains(station_name) {
                    continue;
                }

                // Node representing the current station and line
                let this_node = Node {
                    station: station_name.clone(),
                    line: rail_line.clone()
                };

                // If it's not already in the list of stations, add it
                if !self.stations.contains_key(station_name) {
                    self.stations.insert(station_name.clone(), Vec::new());
                }

                // Connect node instances for different lines at the same station
                // using the correct transfer cost
                let mut node_vec = self.stations.get_mut(station_name).unwrap();
                for existing_node in node_vec.iter() {
                    self.graph.add_edge(existing_node, &this_node, TRANSFER_COST, false);
                }
                node_vec.push(this_node.clone());
                match prev_node {
                    Some(n) => {
                        self.graph.add_edge(&n, &this_node, None, false);
                    },
                    None => {}
                };
                prev_node = Some(this_node)
            }
        }
    }

    /// Rebuild the connections between lines of a particular color
    /// Necessary for the green and red lines
    fn rebuild_connections(&mut self) {
        for &(ref line_one_name, ref line_two_name, ref fallback) in self.connections.iter() {
            // Find the first non-disabled station in line 1
            let line_one = self.source_data.get(line_one_name).unwrap();
            let station_one = match line_one.iter().filter(|&: station| {
                !self.disabled.contains(*station)
            }).next() {
                Some(s) => s,
                None => {
                    // If line 1 has no stations, we don't have a connection to make
                    // ex) if all of the E line is disabled
                    return;
                }
            };

            // Find the first non-disabled station in line 2
            let line_two = self.source_data.get(line_two_name).unwrap();
            let station_two = match line_two.iter().rev().filter(|&: station| {
                !self.disabled.contains(*station)
            }).next() {
                Some(s) => s,
                None => {
                    // If line 2 has no stations, fall back to the optional third line
                    // Disable all B C D, you must connect B to green
                    let fback = match fallback {
                        &Some(ref f) => f.clone(),
                        &None => { return; }
                    };
                    let fallback_line = match self.source_data.get(&fback) {
                        Some(line) => line,
                        None => { return; }
                    };
                    match fallback_line.iter().rev().filter(|&: station| {
                        !self.disabled.contains(*station)
                    }).next() {
                        Some(s) => s,
                        None => { return; }
                    }
                }
            };

            // For the case where we must connect directly to a transfer
            // station due to excess disabling
            let node_vec_one = self.stations.get(station_one).unwrap();
            let node_vec_two = self.stations.get(station_two).unwrap();
            assert!(!node_vec_one.is_empty());
            assert!(!node_vec_two.is_empty());
            for node_one in node_vec_one.iter() {
                for node_two in node_vec_two.iter() {
                    // doesn't matter that we pay the transfer cost here in all cases,
                    // because there is no alternative path to a branch line that avoids
                    // this terminal station connection to the main line
                    self.graph.add_edge(node_one, node_two, TRANSFER_COST, false);
                }
            }
        }
    }


    /// Creates the unbiased nodes used for starting or ending a trip
    /// at a transfer station.
    pub fn add_unbiased_nodes(&mut self) {
        for (station, ref mut node_vec) in self.stations.iter_mut() {
            if node_vec.len() > 1 {
                let start_node = Node {
                    station: station.clone(),
                    line: START_NODE_LABEL.to_string()
                };
                let end_node = Node {
                    station: station.clone(),
                    line: END_NODE_LABEL.to_string()
                };
                for node in node_vec.iter() {
                    self.graph.add_edge(&start_node, node, NO_COST, true);
                    self.graph.add_edge(node, &end_node, NO_COST, true);
                }
                node_vec.push(start_node);
                node_vec.push(end_node);
            }
        }
    }

    /// Find a path from the start to the destination
    pub fn find_path(&self, start: &str, dest: &str) -> TQueryResult {
        let start = return_some_vec!(self.disambiguate_station(start), DisambiguateStart, NoSuchStart);
        let dest = return_some_vec!(self.disambiguate_station(dest), DisambiguateDestination, NoSuchDest);
        let start_node = match self.stations.get(&start) {
            Some(v) => {
                if v.len() == 1 {
                    &v[0]
                } else {
                    &v[v.len() - 2]
                }
            },
            None => { return DisabledStart(start); }
        };
        let dest_node = match self.stations.get(&dest) {
            Some(v) => {
                if v.len() == 1 {
                    &v[0]
                } else {
                    &v[v.len() - 1]
                }
            },
            None => { return DisabledDest(dest); }
        };
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
}

#[cfg(test)]
mod t_tests {
    use super::T;
    use super::{TQueryResult, DisambiguationResult};
    use super::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, NoSuchPath};
    use super::TStep::Station;
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
        let mut stations = t.source_data.get("Braintree").unwrap().iter().chain(
            t.source_data.get("Mattapan").unwrap().iter()).chain(
            t.source_data.get("red").unwrap().iter());
        let mut count: usize = 0;
        for station in stations {
            assert!(expect.contains(station));
            count += 1;
        }
        assert_eq!(count, expect.len());
    }


    #[test]
    fn test_read_connections() {
        let mut t = T::new();
        t.read_connections("data/connections.dat");

        macro_rules! set {
            ($( ($x:expr, $y:expr, $z:expr) ),* ) => {{
                let mut temp_set = HashSet::new();
                $(
                    temp_set.insert(($x.to_string(), $y.to_string(), $z));
                )*
                temp_set
            }};
        }

        let expect = set![("E", "green", None),
                          ("B C D", "green", None),
                          ("B", "B C D", Some("green".to_string())),
                          ("C", "B C D", Some("green".to_string())),
                          ("D", "B C D", Some("green".to_string())),
                          ("Braintree", "red", None),
                          ("Mattapan", "red", None)];
        for connection in t.connections.iter() {
            assert!(expect.contains(connection));
        }
    }

    #[test]
    fn test_rebuild_graph() {
        let mut t = T::new();
        t.load(); // load calls rebuild graph

        assert_eq!(t.stations.len(), 120);

        // disable_station calls rebuild_graph each time
        let mut to_disable = vec![];
        match t.source_data.get("red") {
            Some(stations) => {
                for station in stations.iter() {
                    to_disable.push(station.to_string());
                }
            },
            None => panic!("Bang")
        }

        let mut count = 0;
        for station in to_disable.iter() {
            t.disable_station(station.as_slice());
            count += 1;
        }
        println!("done");

        assert_eq!(t.stations.len(), 120 - count);
    }

    #[test]
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

        let expect3 = DisambiguateDestination(vec!["Government Center Station".to_string(),
                                                   "Hynes Convention Center".to_string(),
                                                   "Malden Center Station".to_string(),
                                                   "Quincy Center Station".to_string(),
                                                   "Tufts Medical Center Station".to_string()]);
        run_find_path_test("South Station", "Center", expect3);
        run_find_path_test("Downtown Crossing Station", "asdf", NoSuchDest);

        let mut t = T::new();
        t.load();
        t.disable_station("Park Street Station");
        t.disable_station("Downtown Crossing Station");
        assert_eq!(t.find_path("Alewife Station", "Ruggles Station"), NoSuchPath);
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
            DisambiguationResult::Station(..) => panic!("Bang!")
        };
        for station in suggestions.iter() {
            assert!(expect.contains(station.as_slice()));
        }
    }
}

/// Interpret the path of Nodes as a list of TSteps
fn interpret_path(path: Vec<Node>) -> Vec<TStep> {
    // invariant: path.len() must be > 0
    assert!(path.len() > 0);
    if path.len() == 1 {
        return Vec::new();
    }

    let mut path_iter = path.into_iter();
    let mut result_vec = Vec::new();
    let first_node = path_iter.next().unwrap();
    let mut prev_node = path_iter.next().unwrap();
    process_first_nodes(&mut result_vec, first_node, prev_node.clone());
    for node in path_iter {
        process_nodes(&mut result_vec, prev_node, node.clone());
        prev_node = node;
    }
    prune_end(&mut result_vec);
    result_vec
}

#[cfg(test)]
mod interpret_path_tests {
    use super::interpret_path;
    use graph::Node;
    use super::TStep::{Station, Switch, Ensure};

    #[test]
    fn test_interpret_path() {
        let mut path = vec![Node {
            station: "Downtown Crossing Station".to_string(),
            line: "orange".to_string()
        }];
        assert_eq!(interpret_path(path.clone()), vec![]);
        path.push(Node {
            station: "Ruggles Station".to_string(),
            line: "orange".to_string()
        });
        let mut expect = vec![Station("Downtown Crossing Station".to_string(),
                                      "orange".to_string()),
                              Station("Ruggles Station".to_string(),
                                      "orange".to_string())];
        assert_eq!(interpret_path(path.clone()), expect);
        path.push(Node {
            station: "Ruggles Station".to_string(),
            line: "blue".to_string()
        });
        assert_eq!(interpret_path(path.clone()), expect);
        path.push(Node {
            station: "State Station".to_string(),
            line: "C".to_string()
        });
        expect.push(Switch("orange".to_string(), "blue".to_string()));
        expect.push(Ensure("C".to_string()));
        expect.push(Station("State Station".to_string(), "C".to_string()));
        assert_eq!(interpret_path(path.clone()), expect);
    }
}

/// returns TSteps associated with a transition between two given nodes
/// EFFECT: mutates steps
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

#[cfg(test)]
mod process_nodes_tests {
    use super::process_nodes;
    use graph::Node;
    use super::TStep::{Station, Switch, Ensure};

    #[test]
    fn test_interpret_path() {
        let prev = Node {
            station: "Downtown Crossing Station".to_string(),
            line: "orange".to_string()
        };
        let curr = Node {
            station: "Ruggles Station".to_string(),
            line: "orange".to_string()
        };
        let mut steps = vec![];
        process_nodes(&mut steps, prev.clone(), curr);
        assert_eq!(steps, vec![Station("Ruggles Station".to_string(),
                                       "orange".to_string())]);
        steps = vec![];
        let curr = Node {
            station: "Downtown Crossing Station".to_string(),
            line: "red".to_string()
        };
        process_nodes(&mut steps, prev.clone(), curr);
        assert_eq!(steps, vec![Switch("orange".to_string(),
                                      "red".to_string())]);
        steps = vec![];
        let curr = Node {
            station: "Ruggles Station".to_string(),
            line: "C".to_string()
        };
        process_nodes(&mut steps, prev.clone(), curr);
        assert_eq!(steps, vec![Ensure("C".to_string()),
                               Station("Ruggles Station".to_string(),
                                       "C".to_string())]);
    }
}

/// Ensure that the first "direction" does not include a Switch
/// or Ensure (due to non-deterministic starting nodes at a transfer station)
/// EFFECT: mutates steps
fn process_first_nodes(steps: &mut Vec<TStep>, prev_node: Node, node: Node) {
    if prev_node.station == node.station {
        steps.push(Station(node.station, node.line));
        return;
    }
    steps.push(Station(prev_node.station.clone(), prev_node.line.clone()));
    process_nodes(steps, prev_node, node);
}

#[cfg(test)]
mod process_first_nodes_tests {
    use super::process_first_nodes;
    use graph::Node;
    use super::TStep::Station;

    #[test]
    fn test_process_first_node() {
        let mut steps = vec![];
        let prev = Node {
            station: "Downtown Crossing Station".to_string(),
            line: "orange".to_string()
        };
        let curr = Node {
            station: "Ruggles Station".to_string(),
            line: "orange".to_string()
        };
        process_first_nodes(&mut steps, prev.clone(), curr);
        assert_eq!(steps, vec![Station("Downtown Crossing Station".to_string(),
                                       "orange".to_string()),
                               Station("Ruggles Station".to_string(),
                                       "orange".to_string())]);
        steps = vec![];
        let curr = Node {
            station: "Downtown Crossing Station".to_string(),
            line: "red".to_string()
        };
        process_first_nodes(&mut steps, prev.clone(), curr);
        assert_eq!(steps, vec![Station("Downtown Crossing Station".to_string(),
                                       "red".to_string())]);

    }
}


/// Ensure that the last "direction" does not include a Switch
/// or Ensure (due to non-deterministic ending nodes at a transfer station)
/// EFFECT: mutates steps
fn prune_end(steps: &mut Vec<TStep>) {
    match steps.pop().unwrap() {
        Station(station, line) => { steps.push(Station(station, line)); },
        _ => {}
    };
}

#[cfg(test)]
mod prune_end_tests {
    use super::prune_end;
    use super::TStep::{Station, Switch, Ensure};

    #[test]
    fn test_prine_end() {
        let mut steps = vec![Station("A".to_string(), "B".to_string())];
        prune_end(&mut steps);
        assert_eq!(steps.len(), 1);

        steps.push(Switch("B".to_string(), "C".to_string()));
        assert_eq!(steps.len(), 2);
        prune_end(&mut steps);
        assert_eq!(steps.len(), 1);

        steps.push(Ensure("B".to_string()));
        assert_eq!(steps.len(), 2);
        prune_end(&mut steps);
        assert_eq!(steps.len(), 1);
    }
}

/// Open the file as given by filename in the form of a Buffered Reader
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}
