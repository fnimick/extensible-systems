use std::io;
use std::io::{TcpListener, TcpStream, Listener, Acceptor};
use std::thread::Thread;
use std::io::net::tcp;

static BIND_ADDR: &'static str = "127.0.0.1:8000";
static INDEX_FILES: [&'static str; 3] = ["index.html", "index.shtml", "index.txt"];

pub fn handle_client(mut stream: TcpStream) {
    let response = b"Hello world.";
    match stream.write(response) {
        Ok(()) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

pub fn serve_forever() {
    let listener = TcpListener::bind(BIND_ADDR).unwrap();
    let mut acceptor = listener.listen().unwrap();
    for stream in acceptor.incoming() {
        match stream {
            Err(e) => {},
            Ok(stream) => {
                Thread::spawn(move || {
                    handle_client(stream)
                });
            }
        }
    }
}
