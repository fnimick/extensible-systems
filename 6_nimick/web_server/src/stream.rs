#[cfg(test)]
use std::io::{Writer, IoResult};

#[doc="
    MemoryStream exists only as an extremely basic testing data structure.
    It is an in-memory data structure that can be read from and written to.
"]

#[cfg(test)]
pub struct MemoryStream {
    read: String,
    write: String,
}

#[cfg(test)]
impl MemoryStream {
    pub fn new(buf: &str) -> MemoryStream {
        MemoryStream {
            read: buf.to_string(),
            write: "".to_string()
        }
    }

    pub fn into_inner(&self) -> (&str, &str) {
        (self.read.as_slice(), self.write.as_slice())
    }
}

#[cfg(test)]
impl Reader for MemoryStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        use std::slice::bytes::copy_memory;
        use std::cmp;

        let buf_len = buf.len();
        let self_len = self.read.len();
        let bytes_read = cmp::min(buf_len, self_len);
        if bytes_read > 0 {
            copy_memory(buf, self.read[0 .. bytes_read].to_string().into_bytes().as_slice());
            self.read = self.read[bytes_read .. self_len].to_string();
        }
        Ok(bytes_read)
    }
}

#[cfg(test)]
impl Writer for MemoryStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.write.push_str(String::from_utf8_lossy(buf).into_owned().as_slice());
        Ok(())
    }
}
