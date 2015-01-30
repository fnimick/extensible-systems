#![allow(unstable)]
use std::os;
use std::io::{File, Open, Read};

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
        let mut file = match File::open_mode(&p, Open, Read) {
            Ok(f) => f,
            Err(e) => panic!("Could not open {}. Error: {}", argument, e),
        };
        match file.read_to_string() {
            Ok(txt) => {
                let (lines, words, chars) = wc(txt);
                println!("{}\t{}\t{}\t{}", lines, words, chars, argument);
            },
            Err(e)  => panic!("Could not read file. Error: {}", e),
        }
    }
}

/*
fn wc(contents: String) -> (usize, usize, usize) {
    let character_count: usize = contents.len();
    let mut word_count: usize = 0;
    let mut line_count: usize = 0;
    for line in contents.as_slice().lines() {
        line_count = line_count + 1;
        let words: Vec<&str> = line.words().collect();
        word_count = word_count + words.len();
    }
    (line_count, word_count, character_count)
}*/

fn wc(contents: String) -> (usize, usize, usize) {
    let mut character_count: usize = 0;
    let mut word_count: usize = 0;
    let mut line_count: usize = 0;
    let mut is_word: bool = false;
    for c in contents.chars() {
        character_count += 1;
        if !c.is_whitespace() {
            if !is_word {
                word_count += 1;
                is_word = true;
            }
        } else {
            is_word = false;
            if c == '\n' {
                line_count += 1;
            }
        }
    }
    (line_count, word_count, character_count)
}

