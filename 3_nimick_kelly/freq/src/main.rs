#![allow(unstable)]
use std::collections::HashMap;
use std::io::BufferedReader;

#[doc="
Determine the word count of the words passed to stdin. Stdin is
considered over when an EOF is reached.

Output one line per word, with its associated word count next to it.
"]
#[cfg(not(test))]
fn main() {
    use std::io;
    use std::io::stdio::StdinReader;

    let stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
}

fn parse_lines<'a, R: Reader>(mut reader: BufferedReader<R>) -> HashMap<&'a str, usize> {
    let mut wordcounts: HashMap<&str, usize> = HashMap::new();
    for line in reader.lines() {
        let l = line.unwrap();
        for word in l.words() {
            inc_count(&mut wordcounts, word.clone());
        }
    }
    wordcounts
}

#[cfg(test)]
mod parse_lines_tests {
    use super::{parse_lines};

    #[test]
    fn tests() {
        let mut expected: HashMap<&str, usize> = HashMap::new();
        expected.insert("Hello", 1);
        expected.insert("World", 2);
        expected.insert("Today", 1);
        expected.insert("is", 1);
        expected.insert("the", 2);
        expected.insert("best", 1);
        expected.insert("day", 1);
        expected.insert("in", 1);
        parse_lines_expect("Hello, World!\nToday is the best day in the World!",
                           expected);
    }

    fn parse_lines_expect(input: &str, expected: HashMap<&str, usize>) {
        let bytes = input.to_string().into_bytes();
        let r: BufferedReader<MemReader> =
            BufferedReader::new(MemReader::new(bytes));
        let output = parse_lines(r);
        assert_eq!(expected, output);
    }
}

fn inc_count<'a>(map: &mut HashMap<&'a str, usize>, word: &'a str) {
    match map.get_mut(word) {
        Some(count) => {*count += 1; return;},
        None => {},
    }
    map.insert(word, 1);
}

#[cfg(test)]
mod inc_count_tests {
    use super::{inc_count};
    use std::collections::HashMap;

    #[test]
    fn test_inc_count() {
        let mut map = HashMap::new();
        inc_count(&mut map, "test");
        inc_count(&mut map, "test");
        inc_count(&mut map, "one");
        assert!(!map.contains_key("nope"));
        assert_eq!(*map.get("test").unwrap(), 2);
        assert_eq!(*map.get("one").unwrap(), 1);
    }
}
