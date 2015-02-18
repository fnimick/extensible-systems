#![allow(unstable)]
use std::collections::HashMap;
use std::io::{File, IoResult, BufferedReader};
use std::os;

#[doc="
    Assumptions: When this program is not given a training corpus,
                   every word is spelled correctly
"]
#[cfg(not(test))]
fn main() {
    use std::io;
    use std::io::stdio::StdinReader;

    let mut args = os::args();
    let training_file = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None       => panic!("Must provide training file")
    };
    let file_reader = open_file(training_file);
    let dictionary = train(file_reader);
    let stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
    correct_spelling(stdin, dictionary);
}

fn open_file(filename: &str) -> BufferedReader<IoResult<File>> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file)
}

fn train<R: Reader>(file: BufferedReader<R>) -> HashMap<String, usize> {
    let x: HashMap<String, usize> = HashMap::new();
    x
}

fn correct_spelling<R: Reader>(words: BufferedReader<R>, dict: HashMap<String, usize>) {
}
