#![allow(unstable)]
#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::io;
use std::io::BufferedReader;
use t::T;
mod t;
mod query;
mod rustyd;
mod files;

fn main() {
    println!("Hello, world!");
    let mut t = T::new();
    t.load();
    rustyd::serve_forever(t);
}

