#![allow(unstable)]
use std::collections::HashMap;


#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
    let mut map = HashMap::new();
    inc_count(&mut map, "test");
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
