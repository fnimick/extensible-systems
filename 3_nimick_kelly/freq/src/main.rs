use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

fn inc_count(map: &mut HashMap<&str, usize>) {
}


#[cfg(test)]
mod freq_tests {
    use super::{inc_count};

    #[test]
    fn test_inc_count() {
        let mut map = HashMap::new::<&str, usize>();
        inc_count(&mut map, &("test"));
        inc_count(&mut map, &("test"));
        inc_count(&mut map, &("one"));
        !assert(!map.contains_key(&("nope")));
        !assert_eq(map.get(&("test")).unwrap(), 2);
        !assert_eq(map.get(&("one")).unwrap(), 1);
    }
}
