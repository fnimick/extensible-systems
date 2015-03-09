use self::FileResult::{FileOk, NotFound, PermissionDenied, FileError};
use std::io::{File, BufferedReader, IoError, IoErrorKind};

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

pub fn open_file(path: &str) -> FileResult {
    match File::open(&Path::new(path)) {
        Ok(f) => FileOk(BufferedReader::new(f)),
        Err(IoError{kind:IoErrorKind::FileNotFound, ..}) => NotFound,
        Err(IoError{kind:IoErrorKind::PermissionDenied, ..}) => PermissionDenied,
        _ => FileError
    }
}

#[cfg(test)]
mod OpenFileTests {
    use super::open_file;
    use super::FileResult;
    #[test]
    fn test_file_not_exist() {
        let my_str = "wharrgarbl";
        match open_file(my_str) {
            FileResult::NotFound => (),
            _ => panic!("bang"),
        }
    }
}
