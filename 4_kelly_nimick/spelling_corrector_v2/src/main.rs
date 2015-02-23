#![allow(unstable)]

#[doc="
Provide spelling corrections on words given via standard input

Words are determined to be spelled correctly by referencing a
training file given to the program as an argument. The more
times a word is used in the training file, the more 'weight'
it's given as 'the word you wanted to spell' - assuming you
input a misspelled word.

Assumptions: The training file has no misspelled words
             A word is only composed of A-Z characters
             A valid word only 1 minor edit away should
               be suggested over a more frequently used word
               two edits away
"]

extern crate regex;

use std::ascii::AsciiExt;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry::{Vacant, Occupied};
use std::io::{File, BufferedReader};
use std::iter::IteratorExt;

static NO_SPELLING_SUGGESTION: &'static str = "-";
static ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";

#[doc="
    Usage: ./spelling_corrector <training_file>

    Words input on standard input will be followed by an output
    in the following format:

    <word>
        * If the word is spelled correctly
    <word>, -
        * If the word is spelled incorrectly, but there are no suggestions
    <word>, <suggestion>
        * If the word is spelled incorrectly, and there is a suggestion
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
        let word = maybe_word.ok().unwrap().to_ascii_lowercase();
        let w = String::from_str(word.trim());
        match suggest(w.clone(), &dictionary) {
            Some(correction) => println!("{}, {}", w, correction),
            None             => println!("{}", w)
        }
    }
}

#[doc="
    Use: string_hash![(&str, value), ... ]
    The &str will be converted into a String value
"]
macro_rules! string_hash {
    ( $( ($x:expr, $y:expr) ),* ) => {{
        let mut temp_hash = HashMap::new();
        $(
            temp_hash.insert(String::from_str($x), $y);
        )*
        temp_hash
    }};
}

#[doc="
    Use: string_set![&str, ... ]
    The &str will be converted into a String value
"]
macro_rules! string_set {
    ( $( $x:expr ),* ) => {{
        let mut temp_set = HashSet::new();
        $(
            temp_set.insert(String::from_str($x));
        )*
        temp_set
    }};
}

/// Open the file as given by filename in the form of a Buffered Reader
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}

/// Remove any preceeding or trailing non a-z or A-Z characters,
/// and return the lowercase version of the word
fn trim_to_word(word: &str) -> Option<String> {
    use regex::Regex;
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

#[cfg(test)]
mod trim_to_word_tests {
    use super::trim_to_word;

    #[test]
    fn tests() {
        test_trim_to_word("hello", "hello");
        test_trim_to_word("Hello,", "hello");
        test_trim_to_word("!Hello,", "hello");
        test_trim_to_word("won't!", "won");
        test_trim_to_word("'won't!'", "won");
        test_trim_to_word("\"Hello,\"", "hello");
        test_trim_to_word("\"Hello,world\"", "hello");
        test_trim_to_word("\"Hello.\"", "hello");
        test_trim_to_word("\"won't''!", "won");
        test_trim_to_word("'fo'c'sle'!", "fo");
    }

    fn test_trim_to_word(check: &str, expect: &str) {
        assert_eq!(trim_to_word(check).unwrap(), expect);
    }
}

/// Given a word and a reference to a HashMap of words to frequencies (usize),
/// increments its associated frequency in the map.
/// If the word is not present, it is added to the map with frequency 1.
fn inc_count(map: &mut HashMap<String, usize>, word: String) {
    match map.entry(word) {
        Vacant(e) => { e.insert(1); },
        Occupied(mut e) => { *e.get_mut() += 1; }
    }
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

#[cfg(test)]
mod train_test {
    use super::train;
    use std::io::{MemReader, BufferedReader};
    use std::collections::HashMap;

    #[test]
    fn test_train() {
        let input = concat!("Hello, World! My name is Frank Underwood.\n",
                            "You may know me as the current president of ",
                            "the United States of America. But I assure ",
                            "you, I am not your typical president. Competence",
                            " is such\n a rare bird in these woods, that I ",
                            "always appreciate it when I see it. You seem ",
                            "bright - maybe there is hope for you after all.");

        let expected = string_hash![("hello", 1), ("world", 1), ("my", 1),
                                    ("name", 1), ("is", 3), ("frank", 1),
                                    ("underwood", 1), ("you", 4), ("may", 1),
                                    ("know", 1), ("me", 1), ("as", 1),
                                    ("the", 2), ("current", 1), ("president", 2),
                                    ("of", 2), ("united", 1), ("states", 1),
                                    ("america", 1), ("but", 1), ("i", 4),
                                    ("assure", 1), ("am", 1), ("not", 1),
                                    ("your", 1), ("typical", 1), ("competence", 1),
                                    ("such", 1), ("a", 1), ("rare", 1),
                                    ("bird", 1), ("in", 1), ("these", 1),
                                    ("woods", 1), ("that", 1), ("always", 1),
                                    ("appreciate", 1), ("it", 2), ("when", 1),
                                    ("see", 1), ("seem", 1), ("bright", 1),
                                    ("maybe", 1), ("there", 1), ("hope", 1),
                                    ("for", 1), ("after", 1), ("all", 1)];
        run_test(input, expected);
    }

    fn run_test(input: &str, expected: HashMap<String, usize>) {
        let bytes = input.to_string().into_bytes();
        let r: BufferedReader<MemReader> =
            BufferedReader::new(MemReader::new(bytes));
        assert_eq!(train(r), expected);
    }
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
        let expect = string_set!["ello", "hllo", "helo", "hell"];
        let hello = String::from_str("hello");
        let input = split_word(&hello);
        assert_eq!(deletions(&input), expect);
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

#[cfg(test)]
mod transpositions_test {
    use super::transpositions;
    use super::split_word;
    use std::collections::HashSet;

    #[test]
    fn test_transpositions() {
        let expect = string_set!["foo", "ofo"];
        let foo = String::from_str("foo");
        let input = split_word(&foo);
        let output = transpositions(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
    }
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
    use super::{split_word, insertions};
    use std::collections::HashSet;

    #[test]
    fn test_insertion() {
        let expect = string_set!["afoo", "bfoo", "cfoo", "dfoo", "efoo", "ffoo",
                 "gfoo", "hfoo", "ifoo", "jfoo", "kfoo", "lfoo", "mfoo", "nfoo",
                 "ofoo", "pfoo", "qfoo", "rfoo", "sfoo", "tfoo", "ufoo", "vfoo",
                 "wfoo", "xfoo", "yfoo", "zfoo", "faoo", "fboo", "fcoo", "fdoo",
                 "feoo", "ffoo", "fgoo", "fhoo", "fioo", "fjoo", "fkoo", "floo",
                 "fmoo", "fnoo", "fooo", "fpoo", "fqoo", "froo", "fsoo", "ftoo",
                 "fuoo", "fvoo", "fwoo", "fxoo", "fyoo", "fzoo", "foao", "fobo",
                 "foco", "fodo", "foeo", "fofo", "fogo", "foho", "foio", "fojo",
                 "foko", "folo", "fomo", "fono", "fooo", "fopo", "foqo", "foro",
                 "foso", "foto", "fouo", "fovo", "fowo", "foxo", "foyo", "fozo",
                 "fooa", "foob", "fooc", "food", "fooe", "foof", "foog", "fooh",
                 "fooi", "fooj", "fook", "fool", "foom", "foon", "fooo", "foop",
                 "fooq", "foor", "foos", "foot", "foou", "foov", "foow", "foox",
                 "fooy", "fooz"];
        let foo = String::from_str("foo");
        let input = split_word(&foo);
        let output = insertions(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
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

#[cfg(test)]
mod replacements_test {
    use super::{split_word, replacements};
    use std::collections::HashSet;

    #[test]
    fn test_replacements() {
        let expect = string_set!["aoo", "boo", "coo", "doo", "eoo", "foo",
            "goo", "hoo", "ioo", "joo", "koo", "loo", "moo", "noo", "ooo",
            "poo", "qoo", "roo", "soo", "too", "uoo", "voo", "woo", "xoo",
            "yoo", "zoo", "fao", "fbo", "fco", "fdo", "feo", "ffo", "fgo",
            "fho", "fio", "fjo", "fko", "flo", "fmo", "fno", "foo", "fpo",
            "fqo", "fro", "fto", "fso", "fuo", "fvo", "fwo", "fxo", "fyo",
            "fzo", "foa", "fob", "foc", "fod", "foe", "fof", "fog", "foh",
            "foi", "foj", "fok", "fol", "fom", "fon", "foo", "fop", "foq",
            "for", "fos", "fot", "fou", "fov", "fow", "fox", "foy", "foz"];
        let foo = String::from_str("foo");
        let input = split_word(&foo);
        let output = replacements(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
    }
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

#[cfg(test)]
mod known_test {
    use super::known;
    use std::collections::{HashSet, HashMap};

    #[test]
    fn test_known() {
        let dict = string_hash![("hello", 2), ("world", 1)];
        let words = string_set!["hello", "word"];
        let expected = string_set!["hello"];
        assert_eq!(known(&words, &dict), expected);
    }
}

/// Given a word, returns a hashmap containing all possible words with edit
/// distance 1 from the given word.
fn edits_1(word: &String) -> HashSet<String> {
    let splits = &split_word(word);
    let results = deletions(splits).into_iter()
        .chain(insertions(splits).into_iter())
        .chain(replacements(splits).into_iter())
        .chain(transpositions(splits).into_iter())
        .collect();
    results
}

#[cfg(test)]
mod edits_1_test {
    use super::{edits_1, split_word, deletions,
        insertions, replacements, transpositions};
    use std::collections::HashSet;

    #[test]
    fn test_edits_1() {
        let foo = String::from_str("foo");
        let word = split_word(&foo);
        let mut expect = HashSet::new();
        expect.extend(deletions(&word).into_iter());
        expect.extend(insertions(&word).into_iter());
        expect.extend(transpositions(&word).into_iter());
        expect.extend(replacements(&word).into_iter());
        let output = edits_1(&foo);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
    }
}

/// Given a set of words with edit distance 1, return a set of words
/// edit distance 2 away from the original source word.
/// Only produces words that are found in the dictionary (to save memory)
fn edits_2(edit_1_set: &HashSet<String>, dict: &HashMap<String, usize>) -> HashSet<String> {
    let mut results = HashSet::new();
    for edit_1 in edit_1_set.iter() {
        results.extend(edits_1(edit_1).into_iter().filter(|w| dict.contains_key(w)))
    }
    results
}

#[cfg(test)]
mod edits_2_test {
    use super::edits_2;
    use std::collections::{HashSet, HashMap};

    #[test]
    fn test_edits_2() {
        let edit_1_set = string_set!["foo"];
        let dict = string_hash![("of", 5), ("food", 3), ("coo", 1),
                                ("roof", 2), ("bar", 1), ("bard", 1)];
        let expect = string_set!["food", "coo"];
        assert_eq!(edits_2(&edit_1_set, &dict), expect);
    }
}

/// Given a word and a dictionary, returns an option:
/// Some(HashSet) if the word is misspelled, with the HashSet
/// giving possible suggestions from edit distance 1 or 2.
/// None if the word is not misspelled.
fn get_suggestion_set(word: String, dict: &HashMap<String, usize>) -> Option<HashSet<String>> {
    let mut word_set = HashSet::new();
    word_set.insert(word.clone());
    let no_change = known(&word_set, dict);
    if !no_change.is_empty() {
        return None
    }
    let one = edits_1(&word);
    let one_known = known(&one, dict);
    Some(if !one_known.is_empty() {
        one_known
    } else {
        edits_2(&one, dict)
    })
}

#[cfg(test)]
mod get_suggestion_set_test {
    use super::get_suggestion_set;
    use std::collections::{HashSet, HashMap};

    #[test]
    fn test_get_suggestion_set() {
        let dict = string_hash![("food", 1), ("room", 1)];
        let expected1 = string_set!["food"];
        let expected2 = string_set!["food", "room"];
        assert_eq!(get_suggestion_set(String::from_str("fo"), &dict), Some(expected1));
        assert_eq!(get_suggestion_set(String::from_str("oo"), &dict), Some(expected2));
        assert_eq!(get_suggestion_set(String::from_str("food"), &dict), None);
    }
}

/// Given a non-empty HashMap and a dictionary,
/// returns the String that represents the best spelling suggestion.
fn get_best_suggestion(corrected_set: HashSet<String>,
                       dict: &HashMap<String, usize>) -> String {
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
    best_word
}

#[cfg(test)]
mod get_best_suggestion_test {
    use super::get_best_suggestion;
    use std::collections::{HashSet, HashMap};

    #[test]
    fn test_get_best_suggestion() {
        let dict = string_hash![("hello", 3), ("hell", 2), ("jello", 1)];
        let suggestions = string_set!["hello", "hell", "jello"];
        assert_eq!(get_best_suggestion(suggestions, &dict),
            String::from_str("hello"));
    }
}

/// Given a word and a dictionary, returns an option:
/// Some(String) if the word is misspelled, with the String indicating the
/// best replacement;
/// None if the word is not misspelled.
fn suggest(word: String, dict: &HashMap<String, usize>) -> Option<String> {
    let mut corrected_set: HashSet<String>;
    match get_suggestion_set(word, dict) {
        Some(set) => { corrected_set = set},
        None => { return None; }
    };

    if corrected_set.is_empty() {
        return Some(String::from_str(NO_SPELLING_SUGGESTION));
    }
    Some(get_best_suggestion(corrected_set, dict))
}

#[cfg(test)]
mod suggest_test {
    use super::{open_file, train, suggest};

    #[test]
    fn test_suggest() {
        let file = open_file("train.txt");
        let dict = train(file);

        let rights = vec!["really", "accomplished", "spelling", "correction", "permanently", "-"];
        let wrongs = vec!["realy", "accomplishher", "spelingg", "correcttio", "permanintly", "wharrgarbl"];

        for (right, wrong) in rights.iter().zip(wrongs.iter()) {
            let w = suggest(String::from_str(*wrong), &dict).unwrap();
            assert_eq!(String::from_str(*right), w);
        }

    }
}

