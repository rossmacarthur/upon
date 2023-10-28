//! Types for value formatters.
//!
//! Value formatters allow you to change the way a [`Value`] is formatted in the
//! rendered template. They can be configured on the engine using
//! [`set_default_formatter`][crate::Engine::set_default_formatter] or
//! [`add_formatter`][crate::Engine::add_formatter].
//!
//! This module defines a [`Formatter`] type that is similar to
//! [`std::fmt::Formatter`] so it should be a familiar API. A mutable reference
//! to this struct is passed to formatter functions and writing to it will
//! update the underlying buffer, be it a [`String`] or an arbitrary
//! [`std::io::Write`] buffer.
//!
//! All formatter functions must have the following signature.
//!
//! ```text
//! use upon::{Value, fmt};
//! Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result;
//! ```
//!
//! Since [`Error`] implements `From<String>` and `From<&str>` it is possible
//! to return custom messages from formatter functions. You can also easily
//! propagate the standard library [`std::fmt::Error`].
//!
//! # Examples
//!
//! ### Escape ASCII
//!
//! Consider a use case where you want to escape all non-ascii characters in
//! strings. We could define a value formatter for that using the standard
//! library function [`escape_ascii`][slice::escape_ascii].
//!
//! ```
//! use std::fmt::Write;
//! use upon::{fmt, Engine, Value};
//!
//! fn escape_ascii(f: &mut fmt::Formatter<'_>, value: &Value) -> fmt::Result {
//!     match value {
//!         Value::String(s) => write!(f, "{}", s.as_bytes().escape_ascii())?,
//!         v => fmt::default(f, v)?, // fallback to default formatter
//!     };
//!     Ok(())
//! }
//!
//! let mut engine = Engine::new();
//! engine.add_formatter("escape_ascii", escape_ascii);
//! ```
//!
//! We could then use this this formatter in templates like this.
//!
//! ```text
//! {{ user.name | escape_ascii }}
//! ```
//!
//! ### Error on [`Value::None`]
//!
//! The [`default`] value formatter formats [`Value::None`] as an empty string.
//! This example demonstrates how you can configure a default formatter to error
//! instead.
//!
//! ```
//! use std::fmt::Write;
//! use upon::{fmt, Engine, Value};
//!
//! fn error_on_none(f: &mut fmt::Formatter<'_>, value: &Value) -> fmt::Result {
//!     match value {
//!         Value::None => Err(fmt::Error::from("unable to format None")),
//!         v => fmt::default(f, v), // fallback to default formatter
//!     }
//! }
//!
//! let mut engine = Engine::new();
//! engine.set_default_formatter(&error_on_none);
//! ```

use std::fmt;
use std::fmt::Write;
use std::io;

use crate::Value;

/// A formatter function or closure.
pub(crate) type FormatFn = dyn Fn(&mut Formatter<'_>, &Value) -> Result + Sync + Send + 'static;

/// A [`std::fmt::Write`] façade.
pub struct Formatter<'a> {
    buf: &'a mut (dyn fmt::Write + 'a),
}

/// The result type returned from a formatter function.
pub type Result = std::result::Result<(), Error>;

/// The error type returned from a formatter function.
#[derive(Debug, Clone)]
pub struct Error(Option<String>);

pub(crate) struct Writer<W> {
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

impl Error {
    pub(crate) fn message(self) -> Option<String> {
        self.0
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(msg) => write!(f, "{msg}"),
            None => write!(f, "format error"),
        }
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self(Some(msg.to_owned()))
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self(Some(msg))
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        Self(None)
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

/// The default value formatter.
///
/// Values are formatted as follows:
/// - [`Value::None`]: empty string
/// - [`Value::Bool`]: `true` or `false`
/// - [`Value::Integer`]: the integer formatted using [`Display`][std::fmt::Display]
/// - [`Value::Float`]: the float formatted using [`Display`][std::fmt::Display]
/// - [`Value::String`]: the string, unescaped
///
/// Errors if the value is a [`Value::List`] or [`Value::Map`].
#[inline]
pub fn default(f: &mut Formatter<'_>, value: &Value) -> Result {
    match value {
        Value::None => {}
        Value::Bool(b) => write!(f, "{b}")?,
        Value::Integer(n) => write!(f, "{n}")?,
        Value::Float(n) => write!(f, "{n}")?,
        Value::String(s) => write!(f, "{s}")?,
        value => {
            return Err(Error::from(format!(
                "expression evaluated to unformattable type {}",
                value.human()
            )));
        }
    }
    Ok(())
}
