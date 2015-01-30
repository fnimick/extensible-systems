#![allow(unstable)]

#[cfg(not(test))]
use std::os;
use std::io::{File, Open, Read};

#[doc = "
Use: ./wc <filename>

This program accepts a filename and calculates the line, word, and character
count output in the following format:

$ wc <filename>
\t<line>\t<word>\t<character> <filename>

Assumptions:
lines must end in '\n'
words are separated by new lines, spaces, or tabs - no other characters
"]

#[cfg(not(test))]
fn main() {
    let mut args = os::args();
    args.remove(0);
    for argument in args.iter() {
        let contents = open_file(argument.as_slice());
        let (lines, words, chars) = wc(contents);
        println!("\t{}\t{}\t{} {}", lines, words, chars, *argument);
    }
}

/// Return the String contents of the file
fn open_file(file: &str) -> String {
    // Verify that it is indeed a file
    let p = Path::new(file);
    let mut file = match File::open_mode(&p, Open, Read) {
        Ok(f) => f,
        Err(e) => panic!("Could not open {}. Error: {}", file, e),
    };
    match file.read_to_string() {
        Ok(txt) => txt,
        Err(e)  => panic!("Could not read file. Error: {}", e),
    }
}

/// Calculate the lines, words, and characters of the given string
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

#[cfg(test)]
mod wc_tests {
    use super::{open_file, wc};

    #[test]
    fn test_wc() {
        assert_eq!((0, 0, 0), wc(strr("")));
        assert_eq!((0, 2, 11), wc(strr("Hello World")));
        assert_eq!((1, 22, 124), wc(strr("Call me Ishmael. I sail the 7 seas hunting that mighty\ttemptress: adventure.\nAhab hunts the\rshe-wolf of the seas: Moby Dick.")));
    }

    #[test]
    #[should_fail]
    fn test_open_nonexistent_file() {
        open_file("nonexistent.txt");
    }

    #[test]
    fn test_bible() {
        assert_eq!((92870, 969905, 5371778), wc(open_file("bible-plain-text.txt")));
    }

    // Because I got tired of typing String::from_str(...)
    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}
