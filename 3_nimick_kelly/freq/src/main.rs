#![allow(unstable)]
extern crate regex;

use std::collections::HashMap;
use std::io::BufferedReader;
use std::ascii::AsciiExt;
use regex::Regex;

#[doc="
Determine the word count of the words passed to stdin.
Stops when an EOF is reached.

Assumptions: Words are compared in a case-insensitive way. Hello == hello.
             Words are composed of characters a-zA-Z.
             Apostrophes are considered part of the word only if there are
             characters a-zA-Z both to the left and to the right.  This allows
             for idiomatic terms such as \"fo'c'sle\".
             Punctuation (excluding apostrophes) does not count as part of the word.

Output one line per word, with its associated word count next to it.
Words are not output in any specified order.
"]
#[cfg(not(test))]
fn main() {
    use std::io;
    use std::io::stdio::StdinReader;

    let stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
    let word_count = parse_lines(stdin);
    for (word, count) in word_count.iter() {
        println!("{}: {}", word, count);
    }
}

/// Remove any preceeding or trailing non a-z or A-Z characters,
/// and truncates words on non-apostrophe punctuation contained within.
fn trim_to_word(word: &str) -> Option<&str> {
    let regex = Regex::new("[a-zA-Z]+(\'[a-zA-Z]+)*");
    let re = match regex {
        Ok(re)    => re,
        Err(..)   => panic!("Could not compile regex")
    };
    match re.captures(word) {
        Some(cap)  => Some(cap.at(0).unwrap()),
        None       => None,
    }
}

#[cfg(test)]
mod trim_to_word_tests {
    use super::trim_to_word;

    #[test]
    fn tests() {
        test_trim_to_word("hello", "hello");
        test_trim_to_word("Hello,", "Hello");
        test_trim_to_word("!Hello,", "Hello");
        test_trim_to_word("won't!", "won't");
        test_trim_to_word("'won't!'", "won't");
        test_trim_to_word("\"Hello,\"", "Hello");
        test_trim_to_word("\"Hello,world\"", "Hello");
        test_trim_to_word("\"Hello.\"", "Hello");
        test_trim_to_word("\"won't''!", "won't");
        test_trim_to_word("\"won't''this!", "won't");
        test_trim_to_word("'fo'c'sle'!", "fo'c'sle");
    }

    fn test_trim_to_word(check: &str, expect: &str) {
        assert_eq!(trim_to_word(check).unwrap(), expect);
    }
}

/// Reads input from BufferedReader and parses individual words,
/// then increments their counts accordingly.
/// Returns a HashMap mapping words to their frequencies.
fn parse_lines<R: Reader>(mut reader: BufferedReader<R>) -> HashMap<String, usize> {
    let mut wordcounts: HashMap<String, usize> = HashMap::new();
    for line in reader.lines() {
        let l = line.unwrap();
        for word in l.words() {
            match trim_to_word(word) {
                Some(w) => inc_count(&mut wordcounts, String::from_str(w)),
                None    => (),
            }
        }
    }
    wordcounts
}

#[cfg(test)]
mod parse_lines_tests {
    use super::{parse_lines};
    use std::collections::HashMap;
    use std::io::{MemReader,BufferedReader};

    #[test]
    fn tests() {
        let mut expected: HashMap<String, usize> = HashMap::new();
        expected.insert(String::from_str("hello"), 1);
        expected.insert(String::from_str("world"), 2);
        expected.insert(String::from_str("today"), 1);
        expected.insert(String::from_str("is"), 1);
        expected.insert(String::from_str("the"), 2);
        expected.insert(String::from_str("best"), 1);
        expected.insert(String::from_str("day"), 1);
        expected.insert(String::from_str("in"), 1);
        expected.insert(String::from_str("whole"), 1);
        expected.insert(String::from_str("wide"), 1);
        parse_lines_expect("Hello, World!\nToday is the best day in the whole-wide World!",
                           expected);
    }

    fn parse_lines_expect(input: &str, expected: HashMap<String, usize>) {
        let bytes = input.to_string().into_bytes();
        let r: BufferedReader<MemReader> =
            BufferedReader::new(MemReader::new(bytes));
        let mut output = parse_lines(r);
        let mut found_keys = Vec::new();
        for (word, count) in output.iter_mut() {
            assert!(expected.contains_key(word));
            match expected.get(word) {
                    Some(expected_count) => assert_eq!(count, expected_count),
                    None                 => assert!(false)
            }
            found_keys.push(word);
        }
    }
}

/// Given a word and a reference to a HashMap of words to frequencies (usize),
/// converts the word to lower case and increments its associated frequency
/// in the map.
/// If the word is not present, it is added to the map with frequency 1.
fn inc_count(map: &mut HashMap<String, usize>, word: String) {
    let lower = word.to_ascii_lowercase();
    match map.get_mut(&lower) {
        Some(count) => {*count += 1; return;},
        None => {},
    }
    map.insert(lower, 1);
}

#[cfg(test)]
mod inc_count_tests {
    use super::{inc_count};
    use std::collections::HashMap;

    #[test]
    fn test_inc_count() {
        let mut map = HashMap::new();
        inc_count(&mut map, String::from_str("test"));
        inc_count(&mut map, String::from_str("Test"));
        inc_count(&mut map, String::from_str("one"));
        assert!(!map.contains_key(&String::from_str("nope")));
        assert_eq!(*map.get(& String::from_str("test")).unwrap(), 2);
        assert_eq!(*map.get(& String::from_str("one")).unwrap(), 1);
    }
}
