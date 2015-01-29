#![allow(unstable)]
use std::os;
use std::io::{BufferedReader, File, Open, Read};

#[doc = "
Use: ./wc <filename>

This program accepts a filename and calculates the line, word, and character
count output in the following format:

$ wc <filename>
<line>\t<word>\t<character>\t<filename>
"]

fn main() {
    let mut args = os::args();
    args.remove(0);
    for argument in args.iter() {
        // Verify that it is indeed a file
        let p = Path::new(argument);
        let file = match File::open_mode(&p, Open, Read) {
            Ok(f) => f,
            Err(e) => panic!("Could not open {}. Error: {}", argument, e),
        };
        let (lines, words, chars) = wc(file);
        println!("{}\t{}\t{}\t{}", lines, words, chars, argument);
    }
}

fn wc(file: File) -> (usize, usize, usize) {
    let mut buf_reader = BufferedReader::new(file);
    let mut character_count: usize = 0;
    let mut word_count: usize = 0;
    let mut line_count: usize = 0;
    loop {
        let line = buf_reader.read_line();
        match line {
            Ok(txt) => {
                line_count = line_count + 1;
                character_count = character_count + txt.len();
                let words: Vec<&str> = txt.words().collect();
                word_count = word_count + words.len();
            },
            Err(..) => { break; },
        }
    }
    (line_count, word_count, character_count)
}
