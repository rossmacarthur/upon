use std::cmp::max;
use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::ast::Span;

/// A convenient type alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur during template compilation or rendering.
#[derive(Clone)]
pub struct Error {
    msg: String,
    tmpl: String,
    span: Span,
}

impl Error {
    pub(crate) fn new(msg: impl Into<String>, tmpl: &str, span: Span) -> Self {
        assert!(!tmpl.is_empty(), "tmpl must be populated");
        Self {
            msg: msg.into(),
            tmpl: tmpl.to_string(),
            span,
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_pretty(self, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt_pretty(self, f)
        } else {
            write!(
                f,
                "{} between bytes {} and {}",
                self.msg, self.span.m, self.span.n
            )
        }
    }
}

fn fmt_pretty(err: &Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let lines: Vec<_> = err.tmpl.split_terminator('\n').collect();
    let (line, col) = to_line_col(&lines, err.span.m);
    let width = max(1, err.tmpl.as_str()[err.span].width());
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
        msg = err.msg
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
