#![allow(unstable)]
#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
use std::io;
use std::io::{File, BufferedReader};
use regex::Regex;
use t::T;
mod t;
mod query;


fn main() {
    println!("Hello, world!");
    let mut stdin = BufferedReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut t = T::new();
    query::query_user(&mut stdout, &mut stdin, &mut t);
}

