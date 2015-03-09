use std::io::{TcpListener, TcpStream, Listener, Acceptor};
use std::io::net::tcp;
use std::thread::Thread;
use std::io::{MemWriter, BufWriter};
use files::FileResult;
use files::FileResult::{FileOk, NotFound, PermissionDenied, FileError};

static HEADER: &'static str = "HTTP/1.0 ";
static CONTENT_TYPE: &'static str = "Content-type: text/";
static CONTENT_LEN: &'static str = "Content-length: ";
static SERVER_NAME: &'static str = "kelly_nimick_web_server";
static BIND_ADDR: &'static str = "127.0.0.1:8000";
static INDEX_FILES: [&'static str; 3] = ["index.html", "index.shtml", "index.txt"];

pub fn handle_client(mut stream: TcpStream, request: FileResult) {
    match stream.write(prepend_response(request, false).get_ref()) {
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
                    handle_client(stream, NotFound)
                });
            }
        }
    }
}

fn prepend_response(response: FileResult, html: bool) -> MemWriter {
    let mut w = MemWriter::with_capacity(HEADER.len() + SERVER_NAME.len());
    w.write_str(HEADER);
    let resp = match response {
        FileOk(..) => "200 OK",
        NotFound => "404 Not Found",
        PermissionDenied => "403 Forbidden",
        _ => "400 Bad Request"
    };
    w.write_line(resp);
    w.write_line(SERVER_NAME);

    match response {
        FileOk(mut buf) => {
            w.write_str(CONTENT_TYPE);
            w.write_line(if html { "html" } else { "plain" });
            w.write_str(CONTENT_LEN);

            let mut file = MemWriter::new();
            while let Ok(o) = buf.read_line() {
                file.write_line(o.as_slice());
            }

            w.write_uint(file.get_ref().len());
            w.write_str("\n");;
            w.write(file.get_ref());
        },
        _ => ()
    };
    w.write_str("\n");

    w
}
