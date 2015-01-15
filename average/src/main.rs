#![allow(unstable)]
use std::io;


#[doc = "
Use: ./average < [data file]

This program accepts a rainfall data file on stdin and provides output on
stdout.

In the rainfall data file, each input line contains one raw measurement.
This measurement is valid if it can be parsed as a 64-bit floating point
number. If the measurement is invalid or is less than 0, it is ignored.
If the measurement is the string '999', average stops consuming input
and prints the output immediately.
If there are no valid measurements, average exits without output.
Otherwise, it prints output on EOF.

The output consists of three lines:
- the average
- the number of measurements in the interval [average,average + 5]
- the number of measurements in the interval [average - 5,average]
"]

fn main() {
    let mut data: Vec<f64> = Vec::new();
    for line in io::stdin().lock().lines() {
        // panics with I/O error if error occurs
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

    let res = average(&data, None);
    match res {
        Some((avg, upper, lower)) => {
            println!("{}", avg);
            println!("{}", upper);
            println!("{}", lower);
        },
        _ => {}
    }

}

#[doc = "
Averages the passed in array of values and provides statistics
about measurements in an interval around the average.

values is a list of measurements, which are floating point numbers
bound defines the interval above and below the average, inclusive,
and defaults to 5.
If values is empty, returns None.
Otherwise, returns Some(tuple):
(the average of the measurements,
 the number of measurements in the interval [average, average + bound],
 the number of measurements in the interval [average - bound, average])
"]
fn average(values: &Vec<f64>, bound: Option<i64>) -> Option<(f64, i64, i64)> {
    if values.len() == 0 {
        return None;
    }
    let my_bound = bound.unwrap_or(5) as f64;
    let mut count = 0i64;
    let mut sum = 0f64;
    for &v in values.iter() {
        sum = sum + v;
        count = count + 1;
    }
    let avg = sum / count as f64;
    let lower_bound = avg - my_bound;
    let upper_bound = avg + my_bound;
    let mut lower = 0i64;
    let mut upper = 0i64;
    for &v in values.iter() {
        if lower_bound < v && v < avg {
            lower += 1;
        }
        if avg < v && v < upper_bound {
            upper += 1;
        }
    }
    Some((avg, upper, lower))
}

#[cfg(test)]
mod average_tests {
    use super::average;

    #[test]
    fn test_average() {
        let v1: Vec<f64> = vec![2f64, 5f64, 10f64, 15f64,
                                0.005f64, 7f64, 3.5f64, 5.5555555f64];
        assert_eq!((6.0075694375, 2, 4), average(&v1, None).unwrap());
    }
}














