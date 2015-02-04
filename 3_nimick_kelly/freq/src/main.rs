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

fn parse_lines<R: Reader>(mut reader: BufferedReader<R>) -> HashMap<String, usize> {
    let mut wordcounts: HashMap<String, usize> = HashMap::new();
    for line in reader.lines() {
        let l = line.unwrap();
        for word in l.words() {
            inc_count(&mut wordcounts, String::from_str(word));
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
        expected.insert(String::from_str("Hello,"), 1);
        expected.insert(String::from_str("World!"), 2);
        expected.insert(String::from_str("Today"), 1);
        expected.insert(String::from_str("is"), 1);
        expected.insert(String::from_str("the"), 2);
        expected.insert(String::from_str("best"), 1);
        expected.insert(String::from_str("day"), 1);
        expected.insert(String::from_str("in"), 1);
        parse_lines_expect("Hello, World!\nToday is the best day in the World!",
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
        /*
        for key in found_keys.iter() {
            output.remove(key);
        }
        assert_eq!(output.len(), 0);
        */
    }
}

fn inc_count(map: &mut HashMap<String, usize>, word: String) {
    match map.get_mut(&word) {
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
        inc_count(&mut map, String::from_str("test"));
        inc_count(&mut map, String::from_str("test"));
        inc_count(&mut map, String::from_str("one"));
        assert!(!map.contains_key(&String::from_str("nope")));
        assert_eq!(*map.get(& String::from_str("test")).unwrap(), 2);
        assert_eq!(*map.get(& String::from_str("one")).unwrap(), 1);
    }
}
