use std::cmp::max;

use crate::fmt;
use crate::types::span::Span;

/// An error that can occur during template compilation or rendering.
pub struct Error {
    /// The type of error, possibly carries a source error.
    kind: ErrorKind,

    /// Optional template name.
    name: Option<String>,

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

    /// A serialization error.
    ///
    /// This can happen when serializing the data to be rendered fails.
    #[cfg(feature = "serde")]
    Serialize,

    /// Rendering failed.
    ///
    /// This can happen for a variety of reasons during rendering. This excludes
    /// function, format and IO errors which are defined below.
    Render,

    /// A function error.
    ///
    /// This can happen if a user defined function returns an error.
    #[cfg(feature = "functions")]
    Function,

    /// A format error.
    ///
    /// This can happen if the expression's value formatter returns an error
    /// while formatting a value into the buffer.
    Format,

    /// An IO error.
    ///
    /// This can only happen when rendering to a type implementing
    /// `std::io::Write` and some IO occurs.
    Io(std::io::Error),
}

impl Error {
    /// Constructs a new syntax error.
    pub(crate) fn syntax(reason: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Syntax,
            name: None,
            reason: Some(reason.into()),
            pretty: Some(Pretty::build(source, span.into())),
        }
    }

    /// Constructs a new render error.
    pub(crate) fn render(reason: impl Into<String>, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Render,
            name: None,
            reason: Some(reason.into()),
            pretty: Some(Pretty::build(source, span.into())),
        }
    }

    /// Constructs a new render error without pretty information.
    #[cfg(feature = "functions")]
    pub(crate) fn render_plain(reason: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Render,
            name: None,
            reason: Some(reason.into()),
            pretty: None,
        }
    }

    /// Constructs a max include depth error.
    pub(crate) fn max_include_depth(max: usize) -> Self {
        Self {
            kind: ErrorKind::Render,
            name: None,
            reason: Some(format!("reached maximum include depth ({max})")),
            pretty: None,
        }
    }

    /// Attaches a template name to the error, if it is not already set.
    pub(crate) fn with_template_name(mut self, name: String) -> Self {
        self.name.get_or_insert(name);
        self
    }

    /// Attaches pretty information to the error.
    #[cfg(feature = "functions")]
    pub(crate) fn enrich(mut self, source: &str, span: impl Into<Span>) -> Self {
        self.pretty
            .get_or_insert_with(|| Pretty::build(source, span.into()));
        self
    }

    #[cfg(feature = "functions")]
    pub(crate) fn function(reason: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Function,
            name: None,
            reason: Some(reason.into()),
            pretty: None,
        }
    }

    pub(crate) fn format(err: fmt::Error, source: &str, span: impl Into<Span>) -> Self {
        Self {
            kind: ErrorKind::Format,
            name: None,
            reason: err.message(),
            pretty: Some(Pretty::build(source, span.into())),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
            name: None,
            reason: None,
            pretty: None,
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Self {
            kind: ErrorKind::Format,
            name: None,
            reason: None,
            pretty: None,
        }
    }
}

#[cfg(feature = "serde")]
#[doc(hidden)]
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            kind: ErrorKind::Serialize,
            name: None,
            reason: Some(msg.to_string()),
            pretty: None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !f.alternate() {
            writeln!(f, "{self:#}")?;
        }

        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("reason", &self.reason)
            .field("pretty", &self.pretty)
            .finish()?;
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match &self.kind {
            ErrorKind::Syntax => "invalid syntax",
            ErrorKind::Render => "render error",
            #[cfg(feature = "functions")]
            ErrorKind::Function => "function error",
            ErrorKind::Format => "format error",
            #[cfg(feature = "serde")]
            ErrorKind::Serialize => "serialize error",
            ErrorKind::Io(_) => "io error",
        };
        match (&self.reason, &self.pretty) {
            (Some(r), Some(p)) if f.alternate() => {
                write!(f, "{msg}")?;
                p.fmt_with_reason(f, self.name.as_deref(), r)
            }
            (Some(reason), _) => write!(f, "{msg}: {reason}"),
            _ => write!(f, "{msg}"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Pretty
////////////////////////////////////////////////////////////////////////////////

/// Holds information necessary for prettily displaying the error.
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

    fn fmt_with_reason(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        name: Option<&str>,
        reason: &str,
    ) -> std::fmt::Result {
        let num = (self.ln + 1).to_string();
        let col = self.col + 1;
        let pad = display_width(&num);
        let align = self.col + self.width;

        let z = "";
        let pipe = "|";
        let equals = "=";
        let underline = "^".repeat(self.width);
        let extra = "-".repeat(3_usize.saturating_sub(self.width));
        let name = name.unwrap_or("<anonymous>");
        let text = &self.text;

        write!(
            f,
            "\n\n {z:pad$}--> {name}:{num}:{col}\
             \n {z:pad$} {pipe}\
             \n {num:>} {pipe} {text}\
             \n {z:pad$} {pipe} {underline:>align$}{extra}\
             \n {z:pad$} {pipe}\
             \n {z:pad$} {equals} reason: {reason}\n",
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
