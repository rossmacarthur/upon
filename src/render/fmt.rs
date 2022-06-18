use std::fmt;
use std::io;

pub struct Formatter<'a> {
    buf: &'a mut (dyn fmt::Write + 'a),
}

pub struct Writer<W> {
    writer: W,
    err: Option<io::Error>,
}

impl<'a> Formatter<'a> {
    pub(crate) fn with_string(buf: &'a mut String) -> Self {
        Self { buf }
    }

    pub(crate) fn with_writer<W>(buf: &'a mut Writer<W>) -> Self
    where
        W: io::Write,
    {
        Self { buf }
    }
}

impl fmt::Write for Formatter<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        fmt::Write::write_str(self.buf, s)
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        fmt::Write::write_char(self.buf, c)
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::Write::write_fmt(self.buf, args)
    }
}

impl<W> Writer<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer, err: None }
    }

    pub fn take_err(&mut self) -> Option<io::Error> {
        self.err.take()
    }
}

impl<W> fmt::Write for Writer<W>
where
    W: io::Write,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_all(s.as_bytes()).map_err(|e| {
            self.err = Some(e);
            fmt::Error
        })
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.writer
            .write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .map_err(|e| {
                self.err = Some(e);
                fmt::Error
            })
    }
}
