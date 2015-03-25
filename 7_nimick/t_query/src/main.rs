#![allow(unstable)]
#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::io;
use std::io::BufferedReader;
use std::io::{TcpListener, Listener, Acceptor, BufferedStream};
use std::sync::{Arc, Mutex};

use t::T;
use query::query_user;

#[cfg(not(test))]
static BIND_ADDR: &'static str = "127.0.0.1:12345";

mod t;
mod query;
mod graph;



//ASSUMPTIONS: don't print when passing through a disabled station
//
//
// TODOS
// fix: file purpose statements
// fix: disable St. Mary

#[cfg(not(test))]
fn main() {
    let mut t = T::new();
    t.load();
    serve_forever(t);
}

/// Start accepting TCP requests and responding to T queries
#[cfg(not(test))]
fn serve_forever(t: T) {
    use std::thread::Thread;

    let mut mbta = Arc::new(Mutex::new(t));

    let listener = TcpListener::bind(BIND_ADDR).unwrap();
    let mut acceptor = listener.listen().unwrap();
    for stream in acceptor.incoming() {
        match stream {
            Err(..) => {},
            Ok(stream) => {
                let tee = mbta.clone();
                Thread::spawn(move || {
                    let mut stream = BufferedStream::new(stream);
                    query_user(&mut stream, tee)
                });
            }
        }
    }
}
