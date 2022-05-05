use std::cmp::max;
use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::span::Span;

/// A convenient type alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur during template compilation or rendering.
#[derive(Clone)]
pub struct Error {
    msg: String,
    span: Option<(String, Span)>,
}

impl Error {
    pub(crate) fn span(msg: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        assert!(!source.is_empty(), "source must be populated");
        Self {
            msg: msg.into(),
            span: Some((source.to_string(), span.into())),
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            msg: msg.to_string(),
            span: None,
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some((source, span)) => fmt_pretty(&self.msg, source, *span, f),
            None => write!(f, "{}", self.msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some((source, span)) => {
                if f.alternate() {
                    fmt_pretty(&self.msg, source, *span, f)
                } else {
                    write!(f, "{} between bytes {} and {}", self.msg, span.m, span.n)
                }
            }
            None => write!(f, "{}", self.msg),
        }
    }
}

fn fmt_pretty(msg: &str, source: &str, span: Span, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        msg = msg
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
