use std::io;
use std::os;
use std::io::{TcpListener, TcpStream, Listener, Acceptor, BufferedStream};
use std::thread::Thread;
use std::io::{MemWriter, BufWriter};
use files::{open_file_with_indices, FileResult};
use files::FileResult::{FileOk, NotFound, PermissionDenied, FileError};

static HEADER: &'static str = "HTTP/1.0 ";
static CONTENT_TYPE: &'static str = "Content-type: text/";
static CONTENT_LEN: &'static str = "Content-length: ";
static SERVER_NAME: &'static str = "kelly_nimick_web_server";
static BIND_ADDR: &'static str = "127.0.0.1:8000";

pub fn handle_client(mut stream: BufferedStream<TcpStream>) {
    let incoming = stream.read_line().unwrap();
    println!("{}", incoming);
    let (request, html) = match get_path(&incoming) {
        Some(path) => {
            println!("{}", path);
            open_file_with_indices(path)
        },
        None => {
            println!("Bad request");
            (FileError, false)
        }
    };
    match stream.write(prepend_response(request, html).get_ref()) {
        Ok(()) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn get_path(s: &String) -> Option<&str> {
    let mut iter = s.words();
    match iter.next() {
        None => return None,
        Some(s) => {
            if s != "GET" {
                return None;
            }
        }
    }
    match iter.next() {
        None => None,
        Some(s) => {
            match s.split(|&: c: char| {c == '?' || c == '#'}).next() {
                Some(r) => {
                    Some(r.slice_from(1))
                },
                _ => None
            }
        }
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
                    handle_client(BufferedStream::new(stream))
                });
            }
        }
    }
}

fn prepend_response(response: FileResult, html: bool) -> MemWriter {
    let mut w = MemWriter::with_capacity(HEADER.len() + SERVER_NAME.len());
    w.write_str(HEADER);
    w.write_line(response.as_str());
    w.write_line(SERVER_NAME);

    match response {
        FileOk(mut buf) => {
            w.write_str(CONTENT_TYPE);
            w.write_line(if html { "html" } else { "plain" });
            w.write_str(CONTENT_LEN);

            let mut file = MemWriter::new();
            while let Ok(o) = buf.read_line() {
                file.write_str(o.as_slice());
            }

            w.write_uint(file.get_ref().len());
            w.write_str("\n\n");
            w.write(file.get_ref());
        },
        _ => ()
    };

    w
}
