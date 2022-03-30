use std::cmp::max;
use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::ast::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    msg: &'static str,
    tmpl: String,
    span: Span,
}

impl Error {
    pub fn new(msg: &'static str, tmpl: &str, span: Span) -> Self {
        Self {
            msg,
            tmpl: tmpl.to_string(),
            span,
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let lines: Vec<_> = self.tmpl.split_terminator('\n').collect();
            let (line, col) = to_line_col(&lines, self.span.m);
            let width = max(1, self.tmpl.as_str()[self.span].width());
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
                msg = self.msg
            )
        } else {
            write!(
                f,
                "{} between bytes {} and {}",
                self.msg, self.span.m, self.span.n
            )
        }
    }
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
