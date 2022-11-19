#![allow(dead_code)]

use std::io;

#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
    count: usize,
    max: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            count: 0,
            max: !0,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            buf: Vec::new(),
            count: 0,
            max,
        }
    }

    #[track_caller]
    pub fn into_string(self) -> String {
        String::from_utf8(self.buf).unwrap()
    }
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += 1;
        if self.count > self.max {
            return Err(io::Error::from(io::ErrorKind::AddrInUse));
        }
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
