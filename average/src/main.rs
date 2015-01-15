#![allow(unstable)]
use std::io;

fn main() {
    let mut stdin = io::stdin();
    let mut lock = stdin.lock();
    let mut lines = lock.lines();
    let mut data = vec![];
    for line in lines {
        let l = line.unwrap();
        let trimmed = l.trim();
        if trimmed == "999" {
            break;
        } else {
            match trimmed.parse::<f64>() {
                Some(x) if x >= 0.0 => data.push(x),
                _ => {}
            }
        }
    }

    let avg = average(&data);

    println!("Average: {}", avg.unwrap());
}

// Averages the array of floats
fn average(data: &Vec<f64>) -> Option<f64> {
    if data.len() == 0 {
        return None;
    }
    let mut count = 0f64;
    let mut sum = 0f64;
    for &d in data.iter() {
        sum = sum + d;
        count = count + 1.0;
    }
    Option::Some(sum / count)
}

#[cfg(test)]
mod average_tests {
    use super::average;

    #[test]
    fn test_average() {
        let v1: Vec<f64> = vec![1f64, 2f64, 3f64];
        assert_eq!(2.0f64, average(&v1).unwrap());
    }
}














