use self::FileResult::{FileOk, NotFound, PermissionDenied, FileError};
use std::io::{File, BufferedReader, IoError, IoErrorKind};

static INDEX_FILES: [&'static str; 3] = ["index.html", "index.shtml", "index.txt"];

pub enum FileResult {
    FileOk(BufferedReader<File>),
    NotFound,
    PermissionDenied,
    FileError,
}

impl FileResult {

    pub fn as_str(&self) -> &str {
        match *self {
            FileOk(..) => "200 OK",
            NotFound => "404 Not Found",
            PermissionDenied => "403 Forbidden",
            FileError => "400 Bad Request"
        }
    }
}

/// If we find PermissionDenied or FileError as the result of opening an index
/// file, then that is returned.
pub fn open_file_with_indices(path: &str) -> (FileResult, bool) {
    if !path.is_empty() && path.chars().rev().next().unwrap() != '/' {
        return (open_file(path.as_slice()), is_html(path.as_slice()));
    }
    for index_file in INDEX_FILES.iter() {
        let index_path = path.clone().to_string() + *index_file;
        match open_file(index_path.as_slice()) {
            NotFound => continue,
            r => return (r, is_html(index_path.as_slice()))
        }
    }
    (NotFound, false)
}

pub fn open_file(path: &str) -> FileResult {
    match File::open(&Path::new(path)) {
        Ok(f) => FileOk(BufferedReader::new(f)),
        Err(IoError{kind:IoErrorKind::FileNotFound, ..}) => NotFound,
        Err(IoError{kind:IoErrorKind::PermissionDenied, ..}) => PermissionDenied,
        _ => FileError
    }
}

fn is_html(s: &str) -> bool {
    s.split('.').rev().next().unwrap() == "html"
}


#[cfg(test)]
mod OpenFileTests {
    use super::{FileResult, open_file};

    #[test]
    fn test_file_not_exist() {
        let my_str = "wharrgarbl".to_string();
        match open_file(&my_str) {
            FileResult::NotFound => (),
            _ => panic!("bang"),
        }
    }

    #[test]
    fn test_file_exists() {
    }
}
