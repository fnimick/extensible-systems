#![allow(unstable)]

#[doc="
Provide spelling corrections on words given via standard input

Words are determined to be spelled correctly by referencing a
training file given to the program as an argument. The more
times a word is used in the training file, the more 'weight'
it's given as 'the word you wanted to spell' - assuming you
input a misspelled word.

Assumptions: When this program is not given a training corpus,
               every word is spelled correctly
             The training file has no misspelled words
             A word is only composed of A-Z characters
"]

extern crate regex;

use regex::Regex;
use std::ascii::AsciiExt;
use std::collections::{HashSet, HashMap};
use std::io::{File, BufferedReader};
use std::iter::IteratorExt;

static NO_SPELLING_SUGGESTION: &'static str = "-";
static ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";

#[doc="
    Usage: ./spelling_corrector <training_file>

    Words input on standard input will be followed by an output
    in the following format:

    <word>
        - If the word is spelled correctly
    <word>: -
        - If the word is spelled incorrectly, but there are no suggestions
    <word>: <suggestion>
        - If the word is spelled incorrectly, and there is a suggestion
"]
#[cfg(not(test))]
fn main() {
    use std::os;
    use std::io;
    use std::io::stdio::StdinReader;

    let args = os::args();
    let training_file = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None       => panic!("Must provide training file")
    };
    let file_reader = open_file(training_file);
    let dictionary = train(file_reader);
    let mut stdin: BufferedReader<StdinReader> = BufferedReader::new(io::stdin());
    for maybe_word in stdin.lines() {
        let word = maybe_word.ok().unwrap();
        match correct(word.clone(), &dictionary) {
            Some(correction) => println!("{}: {}", word, correction),
            None             => println!("{}", word)
        }
    }
}

/// Open the file as given by filename in the form of a Buffered Reader
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
/// Each word in the BufferedReader is counted for frequency
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

/// Given a word, returns a vector containing slices of the word from
/// (0-i, i-<end of word>) for every i from 0 to the word's length.
fn split_word<'a>(word: &'a String) -> Vec<(&'a str, &'a str)> {
    let mut splits = Vec::new();
    let len = word.len();
    for i in range(0, len + 1) {
        splits.push((word.slice(0, i), word.slice(i, len)));
    }
    splits
}

#[cfg(test)]
mod split_word_tests {
    use super::split_word;

    #[test]
    fn test_split_word() {
        let expect = vec![("", "foo"), ("f", "oo"),
                          ("fo", "o"), ("foo", "")];
        let input = String::from_str("foo");
        assert_eq!(split_word(&input), expect);
    }
}

/// Given a split word, returns a HashSet containing all permutations of the
/// word resulting from the deletion of a single letter.
fn deletions(splits: &Vec<(&str, &str)>) -> HashSet<String> {
    splits.iter().filter_map(|&(front, back)| {
        if back.len() > 0 {
            Some(String::from_str(front) + (back.slice_from(1)))
        }
        else { None }
    }).collect()
}

#[cfg(test)]
mod deletions_test {
    use super::deletions;
    use super::split_word;
    use std::collections::HashSet;

    #[test]
    fn test_deletion() {
        let mut expect = HashSet::new();
        expect.insert(strr("ello"));
        expect.insert(strr("hllo"));
        expect.insert(strr("helo"));
        expect.insert(strr("hell"));
        let hello = strr("hello");
        let input = split_word(&hello);
        assert_eq!(deletions(&input), expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}

/// Given a split word, returns a HashSet containing all permutations of the
/// word resulting from the transposition of two adjacent letters.
fn transpositions(splits: &Vec<(&str, &str)>) -> HashSet<String> {
    splits.iter().filter_map(|&(front, back)| {
        if back.len() > 1 {
            let (one, s1) = back.slice_shift_char().unwrap();
            let (two, s2) = s1.slice_shift_char().unwrap();
            let mut s = String::from_str(front);
            s.push(two);
            s.push(one);
            s.push_str(s2);
            Some(s)
        }
        else { None }
    }).collect()
}

/// Given a split word, returns a HashSet containing all permutations of the
/// word resulting from inserting an additional letter at any position.
fn insertions(splits: &Vec<(&str, &str)>) -> HashSet<String> {
    let mut results = HashSet::new();
    for &(front, back) in splits.iter() {
        for c in ALPHABET.chars() {
            let mut s = String::from_str(front);
            s.push(c);
            s.push_str(back);
            results.insert(s);
        }
    }
    results
}

#[cfg(test)]
mod insertions_test {
    use super::insertions;
    use super::split_word;
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
        let foo = strr("foo");
        let input = split_word(&foo);
        assert_eq!(insertions(&input).len(), expect.len());
        assert_eq!(insertions(&input), expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
    }
}


/// Given a split word, returns a HashMap containing all permutations of the
/// word resulting from replacing a letter at any position.
fn replacements(splits: &Vec<(&str, &str)>) -> HashSet<String> {
    let mut results = HashSet::new();
    for &(front, back) in splits.iter() {
        for c in ALPHABET.chars() {
            if back.len() > 0 {
                let mut s = String::from_str(front);
                s.push(c);
                s.push_str(back.slice_from(1));
                results.insert(s);
            }
        }
    }
    results
}

/// Given a set of words, returns a HashSet containing only words that are in
/// the dictionary. If no words are valid, returns an empty HashSet.
fn known(words: &HashSet<String>, dict: &HashMap<String, usize>) -> HashSet<String> {
    let mut recognized = HashSet::new();
    for word in words.iter() {
        if dict.contains_key(word) {
            recognized.insert(word.clone());
        }
    }
    recognized
}

/// Given a word, returns a hashmap containing all possible words with edit
/// distance 1 from the given word.
/// TODO find a better way to do this using collect() ?
fn edits_1(word: &String) -> HashSet<String> {
    let splits = &split_word(word);
    let mut results = HashSet::new();
    for s in deletions(splits).iter()
        .chain(insertions(splits).iter())
        .chain(replacements(splits).iter())
        .chain(transpositions(splits).iter()) {
            results.insert(s.clone());
        }
    results
}

/// Given a set of words with edit distance 1, return a set of words
/// edit distance 2 away from the original source word.
/// Only produces words that are found in the dictionary (to save memory)
fn edits_2(edit_1_set: &HashSet<String>, dict: &HashMap<String, usize>) -> HashSet<String> {
    let mut results = HashSet::new();
    for edit_1 in edit_1_set.iter() {
        for edit_2 in edits_1(edit_1).iter() {
            if dict.contains_key(edit_2) {
                results.insert(edit_2.clone());
            }
        }
    }
    results
}

/// Given a word and a dictionary, returns an option:
/// Some(String) if the word is misspelled, with the String indicating the
/// best replacement;
/// None if the word is not misspelled.
fn correct(word: String, dict: &HashMap<String, usize>) -> Option<String> {

    // Find out if the word is spelled correctly or if there are no suggestions
    let mut corrected_set: HashSet<String>;
    let mut word_set = HashSet::new();
    word_set.insert(word.clone());
    let no_change = known(&word_set, dict);
    if no_change.is_empty() {
        let one = edits_1(&word);
        let one_known = known(&one, dict);
        if one_known.is_empty() {
            let two_known = edits_2(&one, dict);
            if two_known.is_empty() {
                return Some(String::from_str(NO_SPELLING_SUGGESTION));
            }
            corrected_set = two_known;
        } else {
            corrected_set = one_known;
        }
    } else {
        return None;
    }

    // If there are better spelling suggestions, pick the best one
    let mut max_freq: usize = 0;
    let mut best_word = String::new();
    for possibility in corrected_set.into_iter() {
        match dict.get(&possibility) {
            Some(&frequency) => {
                if frequency > max_freq {
                    max_freq = frequency;
                    best_word = possibility;
                }
            },
            None => {}
        }
    }
    Some(best_word)
}

#[cfg(test)]
mod correct_test {
    use super::{open_file, train, correct};

    #[test]
    fn test_correct() {
        let file = open_file("train.txt");
        let dict = train(file);

        let rights = vec!["really", "accomplished", "spelling", "correction", "-"];
        let wrongs = vec!["realy", "accomplishher", "spelingg", "correcttio", "wharrgarbl"];

        for (right, wrong) in rights.iter().zip(wrongs.iter()) {
            let w = correct(String::from_str(*wrong), &dict).unwrap();
            assert_eq!(String::from_str(*right), w);
        }

    }
}
