extern crate regex;

use regex::Regex;
use self::Query::{From, Invalid};

macro_rules! regex (
    ($s:expr) => (regex::Regex::new($s).unwrap());
    );

enum Query<'a> {
    From(&'a str, &'a str),
    Invalid
}

// Does the line match the given regex?
fn parse_line<'a>(line: &'a str, from_regex: &Regex) -> Query<'a> {
    match from_regex.captures(line) {
        Some(cap) => {
            return From(cap.at(1).unwrap().trim(),
                        cap.at(2).unwrap().trim());
        },
        None => {}
    }
    Invalid
}

fn query_user<BS: Writer + Buffer>(stream: &mut BS) {
    let r = regex!(r"from ([a-zA-Z ]+) to ([a-zA-Z ]+)");

    // How can we make this work?
    let parse_line_ = |&: line: &str| -> Query {
        parse_line(line, &r)
    };

    while let Ok(line) = stream.read_line() {
        match parse_line_(line.as_slice()) {
            From(from, to) => {
                // Do stuff
            },
            _ => panic!("bang!")
        }
        stream.flush();
    }
}
























// Listens on a socket and creates a stream to pass to query_user
fn main() {
    use std::io::{TcpListener, Listener, Acceptor, BufferedStream};
    use std::thread::Thread;

    println!("Hello world");
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();
    let mut acceptor = listener.listen().unwrap();
    for stream in acceptor.incoming() {
        match stream {
            Err(..) => {},
            Ok(stream) => {
                Thread::spawn(move || {
                    let mut stream = BufferedStream::new(stream);
                    query_user(&mut stream)
                });
            }
        }
    }
}
