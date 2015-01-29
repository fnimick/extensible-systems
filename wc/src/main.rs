#![allow(unstable)]
use std::os;
use std::io::{File, Open, Read};

#[doc = "
Use: ./wc <filename>

This program accepts a filename and calculates the line, word, and character
count output in the following format:

$ wc <filename>
<line>\t<word>\t<character>\t<filename>
"]

fn main() {
    let mut args = os::args();
    args.remove(0);
    for argument in args.iter() {
        // Verify that it is indeed a file
        let p = Path::new(argument);
        let file = match File::open_mode(&p, Open, Read) {
            Ok(f) => f,
            Err(e) => panic!("Could not open {}. Error: {}", argument, e),
        };
        wc(&file);
    }
}

fn wc(file: &File) {
    println!("{}", file.path().as_str().unwrap());
}
