use std::io;
use std::io::{TcpListener, TcpStream, Listener, Acceptor};
use std::thread::Thread;
use std::io::net::tcp;

static BIND_ADDR: &'static str = "127.0.0.1:8000";
static INDEX_FILES: [&'static str; 3] = ["index.html", "index.shtml", "index.txt"];

pub fn serve_forever() {
    loop {
    }
}
