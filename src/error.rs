use std::cmp::max;
use std::fmt;
use std::io;

use unicode_width::UnicodeWidthStr;

use crate::types::span::Span;

/// An error that can occur during template compilation or rendering.
pub struct Error {
    kind: ErrorKind,
    span: Option<(String, Span)>,
}

#[derive(Debug)]
enum ErrorKind {
    Io(io::Error),
    Fmt(fmt::Error),
    Other(String),
}

impl Error {
    pub(crate) fn new(msg: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Other(msg.into()),
            span: Some((source.to_string(), span.into())),
        }
    }

    pub(crate) fn with_span(mut self, source: &str, span: impl Into<Span>) -> Self {
        self.span = Some((source.to_string(), span.into()));
        self
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
            span: None,
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self {
            kind: ErrorKind::Fmt(err),
            span: None,
        }
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self {
            kind: ErrorKind::Other(msg),
            span: None,
        }
    }
}

#[doc(hidden)]
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            kind: ErrorKind::Other(msg.to_string()),
            span: None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(err) => Some(err),
            ErrorKind::Fmt(err) => Some(err),
            ErrorKind::Other(_) => None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some((source, span)) => fmt_pretty(&self.kind, source, *span, f),
            None => f
                .debug_struct("Error")
                .field("kind", &self.kind)
                .field("span", &self.span)
                .finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some((source, span)) => fmt_pretty(&self.kind, source, *span, f),
            None => fmt::Display::fmt(&self.kind, f),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Io(_) => write!(f, "IO error"),
            ErrorKind::Fmt(_) => write!(f, "format error"),
            ErrorKind::Other(msg) => fmt::Display::fmt(msg, f),
        }
    }
}

fn fmt_pretty(
    kind: &ErrorKind,
    source: &str,
    span: Span,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let lines: Vec<_> = source.split_terminator('\n').collect();
    let (line, col) = to_line_col(&lines, span.m);
    let width = max(1, source[span].width());
    let code = lines.get(line).unwrap_or_else(|| lines.last().unwrap());

    let num = (line + 1).to_string();
    let pad = num.width();
    let pipe = "|";
    let underline = "^".repeat(width);

    write!(
        f,
        "\n \
        {0:pad$} {pipe}\n \
        {num:>} {pipe} {code}\n \
        {0:pad$} {pipe} {underline:>width$} {msg}\n",
        "",
        pad = pad,
        pipe = pipe,
        num = num,
        code = code,
        underline = underline,
        width = col + width,
        msg = kind
    )
}

fn to_line_col(lines: &[&str], offset: usize) -> (usize, usize) {
    let mut n = 0;
    for (i, line) in lines.iter().enumerate() {
        let len = line.width() + 1;
        if n + len > offset {
            return (i, offset - n);
        }
        n += len;
    }
    (lines.len(), lines.last().map(|l| l.width()).unwrap_or(0))
}
