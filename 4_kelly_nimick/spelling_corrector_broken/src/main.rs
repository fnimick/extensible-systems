#![allow(unstable)]
use std::collections::HashMap;
use std::io::BufferedReader;
use std::os;

fn main() {
    let mut args = os::args();
    let train = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None       => panic!("Must provide training file")
    };
    let file_reader = open_file("hello");
    let dictionary = train(file_reader);
}

fn open_file<T>(filename: &str) -> BufferedReader<T> {
    use std::io::File;

    let file = File::open(&Path::new(filename));
    BufferedReader::new(file)
}

fn train<T>(file: BufferedReader<T>) -> HashMap<String, usize> {
    HashMap::new();
}
