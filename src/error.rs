use std::cmp::max;
use std::fmt;
use std::io;

use crate::types::span::Span;

/// An error that can occur during template compilation or rendering.
pub struct Error {
    /// The type of error, possibly carries a source error.
    kind: ErrorKind,

    /// Optional additional reason for this kind of error.
    reason: Option<String>,

    /// Optional pretty information showing the location in the template of the
    /// reason for the error.
    pretty: Option<Pretty>,
}

#[derive(Debug)]
enum ErrorKind {
    /// The template syntax was incorrect.
    ///
    /// This can happen for a variety of reasons if template compilation fails.
    /// The reason field on the parent `Error` will carry more information about
    /// the exact failure.
    Syntax,

    /// Rendering failed.
    ///
    /// This can happen for a variety of reasons during rendering. This excludes
    /// formatting and IO errors and max include depth which are defined below.
    Render,

    /// A serialization error.
    ///
    /// This can happen when serializing the data to be rendered fails.
    Serialize,

    /// An IO error.
    ///
    /// This can only happen when rendering to a type implementing
    /// `std::io::Write` and some IO occurs.
    Io(io::Error),

    /// A format error.
    ///
    /// This can happen if the expression's value formatter returns an error
    /// while formatting a value into the buffer.
    Fmt(fmt::Error),

    /// Any other error, typically constructed from a String in a user defined
    /// source like a filter or formatter.
    Other,
}

impl Error {
    /// Constructs a new syntax error.
    pub(crate) fn syntax(reason: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Syntax,
            reason: Some(reason.into()),
            pretty: Some(Pretty::build(source, span.into())),
        }
    }

    /// Constructs a new render error.
    pub(crate) fn render(reason: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Render,
            reason: Some(reason.into()),
            pretty: Some(Pretty::build(source, span.into())),
        }
    }

    /// Constructs a max include depth error.
    pub(crate) fn max_include_depth(max: usize) -> Self {
        Self {
            kind: ErrorKind::Render,
            reason: Some(format!("reached maximum include depth ({})", max)),
            pretty: None,
        }
    }

    /// Attaches pretty information to the error and converts it to a render
    /// error kind.
    pub(crate) fn into_render(mut self, source: &str, span: impl Into<Span>) -> Self {
        assert!(self.pretty.is_none());
        self.kind = ErrorKind::Render;
        self.pretty = Some(Pretty::build(source, span.into()));
        self
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
            reason: None,
            pretty: None,
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self {
            kind: ErrorKind::Fmt(err),
            reason: None,
            pretty: None,
        }
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self {
            kind: ErrorKind::Other,
            reason: Some(msg),
            pretty: None,
        }
    }
}

#[cfg(feature = "serde")]
#[doc(hidden)]
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            kind: ErrorKind::Serialize,
            reason: Some(msg.to_string()),
            pretty: None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(err) => Some(err),
            ErrorKind::Fmt(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("reason", &self.reason)
            .field("pretty", &self.pretty)
            .finish()?;

        match (&self.pretty, &self.reason) {
            (Some(p), Some(r)) if !f.alternate() => {
                write!(f, "\n\n{}", self.kind)?;
                p.fmt_with_reason(f, r)?
            }
            _ => {}
        }
        Ok(())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        match (&self.pretty, &self.reason) {
            (Some(p), Some(r)) if f.alternate() => p.fmt_with_reason(f, r)?,
            (_, Some(r)) => write!(f, ": {}", r)?,
            _ => {}
        }
        Ok(())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(f, "invalid syntax"),
            Self::Render => write!(f, "failed to render"),
            Self::Serialize => write!(f, "failed to serialize value"),
            Self::Io(_) => write!(f, "io error"),
            Self::Fmt(_) => write!(f, "format error"),
            Self::Other => write!(f, "other error"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Pretty
////////////////////////////////////////////////////////////////////////////////

/// Holds iformation necessary for prettily displaying the error.
#[derive(Debug)]
struct Pretty {
    /// Zero-indexed line number.
    ln: usize,
    /// Zero-indexed column number.
    col: usize,
    /// The number of characters to highlight after `col`.
    width: usize,
    /// The relevant section of template (a single line).
    text: String,
}

impl Pretty {
    fn build(source: &str, span: Span) -> Self {
        let lines: Vec<_> = source.split_terminator('\n').collect();
        let (ln, col) = to_ln_col(&lines, span.m);
        let width = max(1, display_width(&source[span]));
        let text = lines
            .get(ln)
            .unwrap_or_else(|| lines.last().unwrap())
            .to_string();
        Self {
            ln,
            col,
            width,
            text,
        }
    }

    fn fmt_with_reason(&self, f: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        let num = (self.ln + 1).to_string();
        let pad = display_width(&num);
        let pipe = "|";
        let underline = "^".repeat(self.width);
        write!(
            f,
            "\n \
                {0:pad$} {pipe}\n \
                {num:>} {pipe} {text}\n \
                {0:pad$} {pipe} {underline:>width$} {msg}\n",
            "",
            pad = pad,
            pipe = pipe,
            num = num,
            text = self.text,
            underline = underline,
            width = self.col + self.width,
            msg = reason
        )
    }
}

fn to_ln_col(lines: &[&str], offset: usize) -> (usize, usize) {
    let mut n = 0;
    for (i, line) in lines.iter().enumerate() {
        let len = display_width(line) + 1;
        if n + len > offset {
            return (i, offset - n);
        }
        n += len;
    }
    (
        lines.len(),
        lines.last().map(|l| display_width(l)).unwrap_or(0),
    )
}

#[cfg(feature = "unicode")]
fn display_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

#[cfg(not(feature = "unicode"))]
fn display_width(s: &str) -> usize {
    s.chars().count()
}
