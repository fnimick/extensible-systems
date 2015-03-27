#[doc="
    Module: print

    This module is used to output the results of doing operations
    on the T structure
"]

use t::TStep;
use t::TQueryResult;
use t::TOperationResult;
use t::TQueryResult::{TOk, DisambiguateStart, DisambiguateDestination, NoSuchStart, NoSuchDest, DisabledStart, DisabledDest, NoSuchPath};
use t::TOperationResult::{Successful, DisambiguateOp, NoSuchStationOp};
use t::TStep::{Station, Switch, Ensure};

static DISAMBIG_START: &'static str = "disambiguate your start: ";
static DISAMBIG_DEST: &'static str = "disambiguate your destination: ";
static DISAMBIG_OP: &'static str = "disambiguate your target: ";
static SUCCESS_OP: &'static str = "done\n";
static NO_SUCH_START: &'static str = "no such start: ";
static NO_SUCH_DEST: &'static str = "no such destination: ";
static DISABLED_START: &'static str = "disabled start: ";
static DISABLED_DEST: &'static str = "disabled destination: ";
static NO_SUCH_DISABLE: &'static str = "no such station to disable: ";
static NO_SUCH_ENABLE: &'static str = "no such station to enable: ";
static NO_SUCH_PATH: &'static str = "No path exists.\n";

#[allow(unused_must_use)]
/// Print to the output writer the result of calling find_path on the T.
pub fn output_find_path<W: Writer>(path: TQueryResult, from: &str,
                                   to: &str, output: &mut W) {
    match path {
        TOk(steps) => { print_steps(steps, output); },
        DisambiguateStart(suggestions) => { print_vec(DISAMBIG_START, suggestions, output); },
        DisambiguateDestination(suggestions) => { print_vec(DISAMBIG_DEST, suggestions,
                                                            output); },
        NoSuchStart => { print_str(NO_SUCH_START, from, output); },
        NoSuchDest => { print_str(NO_SUCH_DEST, to, output); },
        DisabledStart(s) => { print_str(DISABLED_START, s.as_slice(), output); },
        DisabledDest(s) => { print_str(DISABLED_DEST, s.as_slice(), output); },
        NoSuchPath => { output.write_str(NO_SUCH_PATH); }
    }
}

#[cfg(test)]
mod output_find_path_tests {
    use super::output_find_path;
    use std::io::MemWriter;
    use t::TQueryResult;
    use t::T;

    #[test]
    fn test_output_find_path() {
        let mut t = T::new();
        t.load();

        let (from, to) = ("South Station", "Andrew Station");
        let expect = concat!("South Station, take red\n",
                             "Broadway Station, take red\n",
                             "Andrew Station, take red\n");
        run_test_output_find_path(t.find_path(from, to), from, to, expect);
    }

    /// Test the output of finding a path
    fn run_test_output_find_path(path: TQueryResult,
                                 from: &str, to: &str, expect: &str) {
        let mut w = MemWriter::new();
        output_find_path(path, from, to, &mut w);
        assert_eq!(expect, String::from_utf8(w.into_inner()).unwrap());
    }
}

#[allow(unused_must_use)]
/// Output the result of calling enable or disable a station
fn output_toperation_result<W: Writer>(result: TOperationResult,
                                       station: &str, no_such: &str, output: &mut W) {
    match result {
        Successful => { output.write_str(SUCCESS_OP); },
        DisambiguateOp(suggestions) => { print_vec(DISAMBIG_OP, suggestions, output); },
        NoSuchStationOp => { print_str(no_such, station, output); }
    }
}

#[cfg(test)]
mod output_toperation_result_tests {
    use t::T;
    use t::TOperationResult;
    use std::io::MemWriter;
    use super::{output_enable_station, output_disable_station};
    use super::{NO_SUCH_ENABLE, NO_SUCH_DISABLE, SUCCESS_OP, DISAMBIG_OP};

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
            output_enable_station(station, result, &mut w);
        } else {
            result = t.disable_station(station);
            output_disable_station(station, result, &mut w);
        }

        assert_eq!(expect, String::from_utf8(w.into_inner()).unwrap());
    }
}

/// Print to the output writer the result of enabling the given station
/// Simple wrapper for output_toperation_result
pub fn output_enable_station<W: Writer>(station: &str,
                                        enabled: TOperationResult, output: &mut W) {
    output_toperation_result(enabled, station, NO_SUCH_ENABLE, output)
}

/// Print to the output writer the result of disabling the given station
/// Simple wrapper for output_toperation_result
pub fn output_disable_station<W: Writer>(station: &str,
                                         disabled: TOperationResult, output: &mut W) {
    output_toperation_result(disabled, station, NO_SUCH_DISABLE, output)
}

#[allow(unused_must_use)]
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
    use t::TStep::{Station, Switch, Ensure};
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

#[allow(unused_must_use)]
/// Print the vector of Strings to the writer, preceeded by the header
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

#[allow(unused_must_use)]
/// Print the header and given str to the writer
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
