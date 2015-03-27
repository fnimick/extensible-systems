#![allow(unstable)]
#[doc="
    Module: main

    This module contains the code to load the T data structure and start the
    server that listens for and handles user queries.

    ASSUMPTIONS: don't print when passing through a disabled station
"]
extern crate regex;

#[cfg(not(test))]
use std::io::{TcpListener, Listener, Acceptor, BufferedStream};
#[cfg(not(test))]
use std::sync::{Arc, Mutex};

#[cfg(not(test))]
use t::T;
#[cfg(not(test))]
use query::query_user;

#[cfg(not(test))]
static BIND_ADDR: &'static str = "127.0.0.1:12345";

mod t;
mod query;
mod graph;
mod print;

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

    let mbta = Arc::new(Mutex::new(t));

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
