#![allow(unstable)]
use std::collections::HashMap;
use std::io::BufferedReader;
use std::ascii::AsciiExt;
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
    let train = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None       => panic!("Must provide training file")
    };
    let file_reader = open_file(train);
    let dictionary = train(file_reader);
    let stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
    correct_spelling(stdin, dictionary);
}

fn open_file(filename: &str) -> BufferedReader<Reader> {
    use std::io::File;

    let file = File::open(&Path::new(filename));
    BufferedReader::new(file)
}

fn train<T: Reader>(file: BufferedReader<T>) -> HashMap<String, usize> {
    HashMap::new();
}

fn correct_spelling<T>(words: BufferedReader<T>, dict: HashMap<String, usize>) {
}
