use self::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStation, NoSuchPath};
use self::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use self::TStep::{Station, Switch, Ensure};


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
    NoSuchStation(String),
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

    pub fn find_path(&self, from: &String, to: &String) -> TQueryResult {
        return_some!(self.disambiguate_station(from), DisambiguateStart);
        return_some!(self.disambiguate_station(to), DisambiguateDestination);
        NoSuchPath
    }

    pub fn enable_station(&mut self, station: &String) -> TOperationResult {
        return_some!(self.disambiguate_station(station), DisambiguateOp);
        Successful
    }

    pub fn disable_station(&mut self, station: &String) -> TOperationResult {
        return_some!(self.disambiguate_station(station), DisambiguateOp);
        Successful
    }

    fn disambiguate_station(&self, station: &String) -> Option<Vec<String>> {
        None
    }
}
