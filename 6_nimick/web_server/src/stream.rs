use std::io::{Stream, Buffer, Writer, IoResult, BufferedStream};
use std::slice::bytes::copy_memory;
use std::cmp;

pub struct MemoryStream {
    buf: String
}

impl MemoryStream {
    pub fn new(mem: String) -> MemoryStream {
        MemoryStream {
            buf: mem
        }
    }

    pub fn into_inner(&self) -> &str {
        self.buf.as_slice()
    }
}

impl Reader for MemoryStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let buf_len = buf.len();
        let self_len = self.buf.len();
        let bytes_read = cmp::min(buf_len, self_len);
        if bytes_read > 0 {
            copy_memory(buf, self.buf[0 .. bytes_read].to_string().into_bytes().as_slice());
            self.buf = self.buf[bytes_read - 1 .. self_len].to_string();
        }
        Ok(bytes_read)
    }
}

impl Writer for MemoryStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.buf.push_str(String::from_utf8_lossy(buf).into_owned().as_slice());
        Ok(())
    }
}
