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

    /// Return the HTTP message and code associated with the FileResult
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
/// A borrowed String is passed in rather than a &str, because we are
/// modifying its contents
pub fn open_file_with_indices(path: &String) -> (FileResult, bool) {
    if !path.is_empty() && path.chars().rev().next().unwrap() != '/' {
        return (open_file(path.as_slice()), is_html(path.as_slice()));
    }
    for index_file in INDEX_FILES.iter() {
        let index_path = path.clone() + *index_file;
        match open_file(index_path.as_slice()) {
            NotFound => continue,
            r => return (r, is_html(index_path.as_slice()))
        }
    }
    (NotFound, false)
}

#[cfg(test)]
mod open_file_with_indices_tests {
    use super::{FileResult, open_file_with_indices};

    #[test]
    fn test_file_not_exist() {
        let my_str = "wharrgarbl".to_string();
        match open_file_with_indices(&my_str) {
            (FileResult::NotFound, false) => (),
            _ => panic!("bang"),
        }
    }

    #[test]
    fn test_file_exists() {
        let my_str = "test/index.html".to_string();
        match open_file_with_indices(&my_str) {
            (FileResult::FileOk(..), true) => (),
            _ => panic!("bang"),
        }
    }

    #[test]
    fn test_directory() {
        let my_str = "test/".to_string();
        match open_file_with_indices(&my_str) {
            (FileResult::FileOk(..), true) => (),
            _ => panic!("bang"),
        }
    }
}

/// Open the file at the path given by the input &str
pub fn open_file(path: &str) -> FileResult {
    match File::open(&Path::new(path)) {
        Ok(f) => FileOk(BufferedReader::new(f)),
        Err(IoError{kind:IoErrorKind::FileNotFound, ..}) => NotFound,
        Err(IoError{kind:IoErrorKind::PermissionDenied, ..}) => PermissionDenied,
        _ => FileError
    }
}

#[cfg(test)]
mod open_file_tests {
    use super::{FileResult, open_file};

    #[test]
    fn test_file_not_exist() {
        let my_str = "wharrgarbl";
        match open_file(my_str) {
            FileResult::NotFound => (),
            _ => panic!("bang"),
        }
    }

    #[test]
    fn test_file_exists() {
        let my_str = "test/index.html";
        match open_file(my_str) {
            FileResult::FileOk(..) => (),
            _ => panic!("bang"),
        }
    }
}

/// Determine if the file ends with html
fn is_html(s: &str) -> bool {
    s.split('.').rev().next().unwrap_or("") == "html"
}

#[cfg(test)]
mod is_html_tests {
    use super::is_html;

    #[test]
    fn test_is_html() {
        assert!(is_html("foo/bar/test.html"));
        assert!(!is_html("foo/bar/test.xhtml"));
        assert!(!is_html("!/foo/html/test"));
    }
}
