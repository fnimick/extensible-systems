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

    <word>, <word>
        - If the word is spelled correctly
    <word>, -
        - If the word is spelled incorrectly, but there are no suggestions
    <word>, <suggestion>
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
        let w = String::from_str(word.trim());
        match suggest(w.clone(), &dictionary) {
            Some(correction) => println!("{}, {}", w, correction),
            None             => println!("{}", w)
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
        let mut expected = HashMap::new();
        expected.insert(strr("hello"), 1);
        expected.insert(strr("world"), 1);
        expected.insert(strr("my"), 1);
        expected.insert(strr("name"), 1);
        expected.insert(strr("is"), 3);
        expected.insert(strr("frank"), 1);
        expected.insert(strr("underwood"), 1);
        expected.insert(strr("you"), 4);
        expected.insert(strr("may"), 1);
        expected.insert(strr("know"), 1);
        expected.insert(strr("me"), 1);
        expected.insert(strr("as"), 1);
        expected.insert(strr("the"), 2);
        expected.insert(strr("current"), 1);
        expected.insert(strr("president"), 2);
        expected.insert(strr("of"), 2);
        expected.insert(strr("united"), 1);
        expected.insert(strr("states"), 1);
        expected.insert(strr("america"), 1);
        expected.insert(strr("but"), 1);
        expected.insert(strr("i"), 4);
        expected.insert(strr("assure"), 1);
        expected.insert(strr("am"), 1);
        expected.insert(strr("not"), 1);
        expected.insert(strr("your"), 1);
        expected.insert(strr("typical"), 1);
        expected.insert(strr("competence"), 1);
        expected.insert(strr("such"), 1);
        expected.insert(strr("a"), 1);
        expected.insert(strr("rare"), 1);
        expected.insert(strr("bird"), 1);
        expected.insert(strr("in"), 1);
        expected.insert(strr("these"), 1);
        expected.insert(strr("woods"), 1);
        expected.insert(strr("that"), 1);
        expected.insert(strr("always"), 1);
        expected.insert(strr("appreciate"), 1);
        expected.insert(strr("it"), 2);
        expected.insert(strr("when"), 1);
        expected.insert(strr("see"), 1);
        expected.insert(strr("seem"), 1);
        expected.insert(strr("bright"), 1);
        expected.insert(strr("maybe"), 1);
        expected.insert(strr("there"), 1);
        expected.insert(strr("hope"), 1);
        expected.insert(strr("for"), 1);
        expected.insert(strr("after"), 1);
        expected.insert(strr("all"), 1);
        run_test(input, expected);
    }

    fn run_test(input: &str, expected: HashMap<String, usize>) {
        let bytes = input.to_string().into_bytes();
        let r: BufferedReader<MemReader> =
            BufferedReader::new(MemReader::new(bytes));
        assert_eq!(train(r), expected);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
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

#[cfg(test)]
mod transpositions_test {
    use super::transpositions;
    use super::split_word;
    use std::collections::HashSet;

    #[test]
    fn test_transpositions() {
        let mut expect = HashSet::new();
        expect.insert(strr("foo"));
        expect.insert(strr("ofo"));
        let foo = strr("foo");
        let input = split_word(&foo);
        let output = transpositions(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
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
        let output = insertions(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
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

#[cfg(test)]
mod replacements_test {
    use super::replacements;
    use super::split_word;
    use std::collections::HashSet;

    #[test]
    fn test_replacements() {
        let mut expect = HashSet::new();
        expect.insert(strr("aoo"));
        expect.insert(strr("boo"));
        expect.insert(strr("coo"));
        expect.insert(strr("doo"));
        expect.insert(strr("eoo"));
        expect.insert(strr("foo"));
        expect.insert(strr("goo"));
        expect.insert(strr("hoo"));
        expect.insert(strr("ioo"));
        expect.insert(strr("joo"));
        expect.insert(strr("koo"));
        expect.insert(strr("loo"));
        expect.insert(strr("moo"));
        expect.insert(strr("noo"));
        expect.insert(strr("ooo"));
        expect.insert(strr("poo"));
        expect.insert(strr("qoo"));
        expect.insert(strr("roo"));
        expect.insert(strr("soo"));
        expect.insert(strr("too"));
        expect.insert(strr("uoo"));
        expect.insert(strr("voo"));
        expect.insert(strr("woo"));
        expect.insert(strr("xoo"));
        expect.insert(strr("yoo"));
        expect.insert(strr("zoo"));
        expect.insert(strr("fao"));
        expect.insert(strr("fbo"));
        expect.insert(strr("fco"));
        expect.insert(strr("fdo"));
        expect.insert(strr("feo"));
        expect.insert(strr("ffo"));
        expect.insert(strr("fgo"));
        expect.insert(strr("fho"));
        expect.insert(strr("fio"));
        expect.insert(strr("fjo"));
        expect.insert(strr("fko"));
        expect.insert(strr("flo"));
        expect.insert(strr("fmo"));
        expect.insert(strr("fno"));
        expect.insert(strr("foo"));
        expect.insert(strr("fpo"));
        expect.insert(strr("fqo"));
        expect.insert(strr("fro"));
        expect.insert(strr("fso"));
        expect.insert(strr("fto"));
        expect.insert(strr("fuo"));
        expect.insert(strr("fvo"));
        expect.insert(strr("fwo"));
        expect.insert(strr("fxo"));
        expect.insert(strr("fyo"));
        expect.insert(strr("fzo"));
        expect.insert(strr("foa"));
        expect.insert(strr("fob"));
        expect.insert(strr("foc"));
        expect.insert(strr("fod"));
        expect.insert(strr("foe"));
        expect.insert(strr("fof"));
        expect.insert(strr("fog"));
        expect.insert(strr("foh"));
        expect.insert(strr("foi"));
        expect.insert(strr("foj"));
        expect.insert(strr("fok"));
        expect.insert(strr("fol"));
        expect.insert(strr("fom"));
        expect.insert(strr("fon"));
        expect.insert(strr("foo"));
        expect.insert(strr("fop"));
        expect.insert(strr("foq"));
        expect.insert(strr("for"));
        expect.insert(strr("fos"));
        expect.insert(strr("fot"));
        expect.insert(strr("fou"));
        expect.insert(strr("fov"));
        expect.insert(strr("fow"));
        expect.insert(strr("fox"));
        expect.insert(strr("foy"));
        expect.insert(strr("foz"));
        let foo = strr("foo");
        let input = split_word(&foo);
        let output = replacements(&input);
        assert_eq!(output.len(), expect.len());
        assert_eq!(output, expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
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
        let mut dict = HashMap::new();
        dict.insert(strr("hello"), 2);
        dict.insert(strr("world"), 1);
        let mut words = HashSet::new();
        words.insert(strr("hello"));
        words.insert(strr("word"));
        let mut expected = HashSet::new();
        expected.insert(strr("hello"));
        assert_eq!(known(&words, &dict), expected);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
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
        let foo = strr("foo");
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

    fn strr(string: &str) -> String {
        String::from_str(string)
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
        let mut edit_1_set = HashSet::new();
        edit_1_set.insert(strr("foo"));
        let mut dict = HashMap::new();
        dict.insert(strr("of"), 5);
        dict.insert(strr("food"), 3);
        dict.insert(strr("coo"), 1);
        dict.insert(strr("roof"), 2);
        dict.insert(strr("bar"), 1);
        dict.insert(strr("bard"), 1);
        let mut expect = HashSet::new();
        expect.insert(strr("food"));
        expect.insert(strr("coo"));
        assert_eq!(edits_2(&edit_1_set, &dict), expect);
    }

    fn strr(string: &str) -> String {
        String::from_str(string)
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

        let rights = vec!["really", "accomplished", "spelling", "correction", "perminantly", "-"];
        let wrongs = vec!["realy", "accomplishher", "spelingg", "correcttio", "permanently", "wharrgarbl"];

        for (right, wrong) in rights.iter().zip(wrongs.iter()) {
            let w = suggest(String::from_str(*wrong), &dict).unwrap();
            assert_eq!(String::from_str(*right), w);
        }

    }
}
