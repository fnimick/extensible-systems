#[cfg(not(test))]
use std::io::{TcpListener, Listener, Acceptor, BufferedStream};

use std::sync::{Arc, Mutex};
use std::io::MemWriter;
use files::{open_file_with_indices, FileResult};
use files::FileResult::{FileOk, BadRequest};
use query::query_user;
use t::T;

static HEADER: &'static str = "HTTP/1.0 ";
static CONTENT_TYPE: &'static str = "Content-type: text/";
static CONTENT_LEN: &'static str = "Content-length: ";
static SERVER_NAME: &'static str = "kelly_nimick_web_server";

#[cfg(not(test))]
static BIND_ADDR: &'static str = "127.0.0.1:12345";

/// Accept an incoming client stream and respond to its request
pub fn handle_client<BS: Buffer + Writer>(stream: &mut BS, t: Arc<Mutex<T>>) {
    query_user(stream, t);
    /*
    let incoming = stream.read_line().unwrap();
    println!("{}", incoming);
    let (request, html) = match get_path(incoming.as_slice()) {
        Some(path) => {
            println!("{}", path);
            open_file_with_indices(path)
        },
        None => {
            println!("Bad request");
            (BadRequest, false)
        }
    };
    match stream.write(prepend_response(request, html).get_ref()) {
        Ok(()) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
    */
}

#[cfg(test)]
mod handle_client_tests {
    use super::{prepend_response, handle_client};
    use std::io::BufferedStream;
    use files::open_file;
    use stream::MemoryStream;

    #[test]
    fn test_handle_client() {
        let request = "GET /test/index.txt\n";
        let stream = MemoryStream::new(request);
        let mut s = BufferedStream::new(stream);
        handle_client(&mut s);
        let expect = String::from_utf8(prepend_response(
                open_file("test/index.txt"), false).into_inner()).ok().unwrap();
        assert_eq!(s.into_inner().into_inner().1, expect);
    }
}

/// Get the pathname associated with the HTTP request
fn get_path(s: &str) -> Option<&str> {
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

#[cfg(test)]
mod get_path_tests {
    use super::get_path;

    #[test]
    fn test_get_path() {
        assert_eq!(get_path("GET /foo.html").unwrap(), "foo.html");
        assert_eq!(get_path("GET /foo.html?query=bar").unwrap(), "foo.html");
        assert_eq!(get_path("GET /foo.html#hash").unwrap(), "foo.html");
        assert_eq!(get_path("GET /test/foo.html#hash").unwrap(), "test/foo.html");
        assert_eq!(get_path("HEAD /foo.html#hash"), None);
        assert_eq!(get_path(""), None);
    }
}

/// Start accepting TCP requests and responding to HTTP/0.9 requests
#[cfg(not(test))]
pub fn serve_forever(t: T) {
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
                    handle_client(&mut stream, tee)
                });
            }
        }
    }
}

/// Add the HTTP/0.9 headers to the output
#[allow(unused_must_use)]
fn prepend_response(response: FileResult, html: bool) -> MemWriter {
    let mut w = MemWriter::with_capacity(HEADER.len() + SERVER_NAME.len());
    w.write_str(HEADER);
    w.write_line(response.as_str());
    w.write_line(SERVER_NAME);
    w.write_str(CONTENT_TYPE);
    w.write_line(if html { "html" } else { "plain" });
    w.write_str(CONTENT_LEN);

    match response {
        FileOk(mut buf) => {
            let mut file = MemWriter::new();
            while let Ok(o) = buf.read_line() {
                file.write_str(o.as_slice());
            }

            w.write_uint(file.get_ref().len());
            w.write_str("\n\n");
            w.write(file.get_ref());
        },
        _ => {
            w.write_uint(0);
            w.write_str("\n\n");
        }
    };

    w
}
