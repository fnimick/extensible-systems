#![allow(unstable)]
extern crate regex;

use std::char;
use regex::Regex;
use std::ascii::AsciiExt;
use std::collections::{HashSet, HashMap};
use std::io::{File, IoResult, BufferedReader};
use std::num::ToPrimitive;

static NO_SPELLING_SUGGESTION: &'static str = "-";

#[doc="
    Assumptions: When this program is not given a training corpus,
                   every word is spelled correctly
"]
#[cfg(not(test))]
fn main() {
    use std::os;
    use std::io;
    use std::io::stdio::StdinReader;

    let mut args = os::args();
    let training_file = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None       => panic!("Must provide training file")
    };
    let file_reader = open_file(training_file);
    let dictionary = train(file_reader);
    let mut stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
    for maybe_word in stdin.lines() {
        let word = maybe_word.ok().unwrap();
        let spellchecked: String = correct_spelling(word.clone(), &dictionary);
        println!("{}: {}", word, spellchecked);
    }
}

fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}

/// Remove any preceeding or trailing non a-z or A-Z characters,
/// and return the lowercase version of the word
fn trim_to_word(word: &str) -> Option<String> {
    let regex = Regex::new("[a-zA-Z]+");
    let re = match regex {
        Ok(re)    => re,
        Err(..)   => panic!("Could not compile regex")
    };
    match re.captures(word) {
        Some(cap)  => Some(cap.at(0).unwrap().to_ascii_lowercase()),
        None       => None,
    }
}

/// Given a word and a reference to a HashMap of words to frequencies (usize),
/// converts the word to lower case and increments its associated frequency
/// in the map.
/// If the word is not present, it is added to the map with frequency 1.
fn inc_count(map: &mut HashMap<String, usize>, word: String) {
    match map.get_mut(&word) {
        Some(count) => {*count += 1; return;},
        None => {},
    }
    map.insert(word, 1);
}

/// Train the program to identify words based on the corpus of passed-in data
/// The data in the BufferedReader is read and counted
fn train<R: Reader>(mut file: BufferedReader<R>) -> HashMap<String, usize> {
    let mut x: HashMap<String, usize> = HashMap::new();

    for line in file.lines() {
        for word in line.unwrap().words() {
            match trim_to_word(word.as_slice()) {
                Some(w) => inc_count(&mut x, w),
                None    => {}
            }
        }
    }
    x
}

fn correct_spelling(word: String, dict: &HashMap<String, usize>) -> String {
    match dict.get(word.as_slice()) {
        Some(..) => word,
        None     => {
            let mut most_likely_word_count: usize = 0;
            let mut most_likely_word = String::from_str(NO_SPELLING_SUGGESTION);
            let permutations = match trim_to_word(word.as_slice()) {
                Some(w) => variations(w, dict),
                None    => panic!("No word found")
            };
            for w in permutations.iter() {
                match dict.get(w.as_slice()) {
                    Some(&count) => {
                        if count > most_likely_word_count {
                            most_likely_word_count = count;
                            most_likely_word = w.clone();
                        }
                    },
                    None        => {}
                }
            }
            most_likely_word
        }
    }
}

#[cfg(test)]
mod test_correct_spelling {
    use super::correct_spelling;
    use std::collections::HashMap;

    #[test]
    fn test_spelling() {
        let mut dict = HashMap::new();
        dict.insert(strr("hell"), 2);
        dict.insert(strr("hello"), 1);
        dict.insert(strr("word"), 3);
        dict.insert(strr("world"), 1);
        dict.insert(strr("race"), 3);
        dict.insert(strr("acer"), 1);
        assert_eq!(correct_spelling(strr("hello"), &dict), strr("hello"));
        assert_eq!(correct_spelling(strr("hellp"), &dict), strr("hell"));
        assert_eq!(correct_spelling(strr("worldc"), &dict), strr("world"));
        assert_eq!(correct_spelling(strr("worod"), &dict), strr("word"));
        assert_eq!(correct_spelling(strr("racer"), &dict), strr("race"));
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}

fn variations(word: String, dict: & HashMap<String, usize>) -> HashSet<String> {
    delete_letter(word)
}

fn delete_letter(word: String) -> HashSet<String> {
    let mut variations = HashSet::new();

    // Delete one letter
    for i in 0..word.len() {
        let mut w = word.clone();
        w.remove(i);
        variations.insert(w);
    }
    variations
}

#[cfg(test)]
mod delete_letter_test {
    use super::delete_letter;
    use std::collections::HashSet;

    #[test]
    fn test_deletion() {
        let mut expect = HashSet::new();
        expect.insert(strr("ello"));
        expect.insert(strr("hllo"));
        expect.insert(strr("helo"));
        expect.insert(strr("hell"));
        assert_eq!(delete_letter(strr("hello")), expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}

fn insert_letter(word: String) -> HashSet<String> {
    let mut variations = HashSet::new();

    // Insert one letter
    let a = 97;
    let z = a + 26;
    for l in 0..(z-a) {
        for i in 0..word.len()+1 {
            let mut w = word.clone();
            let c = char::from_u32(a + l).unwrap();
            w.insert(i, c);
            variations.insert(w);
        }
    }
    variations
}

#[cfg(test)]
mod insert_letter_test {
    use super::insert_letter;
    use std::collections::HashSet;

    #[test]
    fn test_insertion() {
        let mut expect = HashSet::new();
        expect.insert(strr("afoo"));
        expect.insert(strr("bfoo"));
        expect.insert(strr("cfoo"));
        expect.insert(strr("dfoo"));
        expect.insert(strr("efoo"));
        expect.insert(strr("ffoo"));
        expect.insert(strr("gfoo"));
        expect.insert(strr("hfoo"));
        expect.insert(strr("ifoo"));
        expect.insert(strr("jfoo"));
        expect.insert(strr("kfoo"));
        expect.insert(strr("lfoo"));
        expect.insert(strr("mfoo"));
        expect.insert(strr("nfoo"));
        expect.insert(strr("ofoo"));
        expect.insert(strr("pfoo"));
        expect.insert(strr("qfoo"));
        expect.insert(strr("rfoo"));
        expect.insert(strr("sfoo"));
        expect.insert(strr("tfoo"));
        expect.insert(strr("ufoo"));
        expect.insert(strr("vfoo"));
        expect.insert(strr("wfoo"));
        expect.insert(strr("xfoo"));
        expect.insert(strr("yfoo"));
        expect.insert(strr("zfoo"));
        expect.insert(strr("faoo"));
        expect.insert(strr("fboo"));
        expect.insert(strr("fcoo"));
        expect.insert(strr("fdoo"));
        expect.insert(strr("feoo"));
        expect.insert(strr("ffoo"));
        expect.insert(strr("fgoo"));
        expect.insert(strr("fhoo"));
        expect.insert(strr("fioo"));
        expect.insert(strr("fjoo"));
        expect.insert(strr("fkoo"));
        expect.insert(strr("floo"));
        expect.insert(strr("fmoo"));
        expect.insert(strr("fnoo"));
        expect.insert(strr("fooo"));
        expect.insert(strr("fpoo"));
        expect.insert(strr("fqoo"));
        expect.insert(strr("froo"));
        expect.insert(strr("fsoo"));
        expect.insert(strr("ftoo"));
        expect.insert(strr("fuoo"));
        expect.insert(strr("fvoo"));
        expect.insert(strr("fwoo"));
        expect.insert(strr("fxoo"));
        expect.insert(strr("fyoo"));
        expect.insert(strr("fzoo"));
        expect.insert(strr("foao"));
        expect.insert(strr("fobo"));
        expect.insert(strr("foco"));
        expect.insert(strr("fodo"));
        expect.insert(strr("foeo"));
        expect.insert(strr("fofo"));
        expect.insert(strr("fogo"));
        expect.insert(strr("foho"));
        expect.insert(strr("foio"));
        expect.insert(strr("fojo"));
        expect.insert(strr("foko"));
        expect.insert(strr("folo"));
        expect.insert(strr("fomo"));
        expect.insert(strr("fono"));
        expect.insert(strr("fooo"));
        expect.insert(strr("fopo"));
        expect.insert(strr("foqo"));
        expect.insert(strr("foro"));
        expect.insert(strr("foso"));
        expect.insert(strr("foto"));
        expect.insert(strr("fouo"));
        expect.insert(strr("fovo"));
        expect.insert(strr("fowo"));
        expect.insert(strr("foxo"));
        expect.insert(strr("foyo"));
        expect.insert(strr("fozo"));
        expect.insert(strr("fooa"));
        expect.insert(strr("foob"));
        expect.insert(strr("fooc"));
        expect.insert(strr("food"));
        expect.insert(strr("fooe"));
        expect.insert(strr("foof"));
        expect.insert(strr("foog"));
        expect.insert(strr("fooh"));
        expect.insert(strr("fooi"));
        expect.insert(strr("fooj"));
        expect.insert(strr("fook"));
        expect.insert(strr("fool"));
        expect.insert(strr("foom"));
        expect.insert(strr("foon"));
        expect.insert(strr("fooo"));
        expect.insert(strr("foop"));
        expect.insert(strr("fooq"));
        expect.insert(strr("foor"));
        expect.insert(strr("foos"));
        expect.insert(strr("foot"));
        expect.insert(strr("foou"));
        expect.insert(strr("foov"));
        expect.insert(strr("foow"));
        expect.insert(strr("foox"));
        expect.insert(strr("fooy"));
        expect.insert(strr("fooz"));
        assert_eq!(insert_letter(strr("foo")), expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}




