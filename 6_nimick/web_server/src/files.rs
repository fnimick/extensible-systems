use self::FileResult::{FileOk, NotFound, FileError};
use std::io::{File, BufferedReader, IoError, IoErrorKind};

pub enum FileResult {
    FileOk(BufferedReader<File>),
    NotFound,
    FileError,
}


pub fn open_file(path: &str) -> FileResult {
    match File::open(&Path::new(path)) {
        Ok(f) => FileOk(BufferedReader::new(f)),
        Err(IoError{kind:IoErrorKind::FileNotFound, ..}) => NotFound,
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
