#![allow(clippy::wrong_self_convention)]

mod core;
mod iter;
mod stack;
mod value;

use std::io;

use crate::fmt::{Formatter, Writer};
#[cfg(feature = "filters")]
pub use crate::render::core::FilterState;
use crate::render::core::RendererImpl;
pub use crate::render::stack::Stack;
use crate::types::program::Template;
use crate::{Engine, Error, Result, Value, ValueFn};

fn to_string<'a>(
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
    stack: Stack<'a>,
) -> Result<String> {
    let mut s = String::with_capacity(template.source.len());
    let mut f = Formatter::with_string(&mut s);
    RendererImpl {
        engine,
        template,
        stack,
    }
    .render(&mut f)?;
    Ok(s)
}

fn to_writer<'a, W>(
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
    stack: Stack<'a>,
    writer: W,
) -> Result<()>
where
    W: io::Write,
{
    let mut w = Writer::new(writer);
    let mut f = Formatter::with_writer(&mut w);
    RendererImpl {
        engine,
        template,
        stack,
    }
    .render(&mut f)
    .map_err(|err| w.take_err().map(Error::from).unwrap_or(err))
}

/// A renderer that interprets a compiled [`Template`][crate::Template].
///
/// This struct is created by one of the following functions:
/// - [`Template{,Ref}::render`][crate::Template::render]
/// - [`Template{,Ref}::render_from`][crate::Template::render_from]
/// - [`Template{,Ref}::render_from_fn`][crate::Template::render_from_fn]
#[cfg(feature = "serde")]
#[must_use = "must call `.to_string()` or `.to_writer(..)` on the renderer"]
pub struct Renderer<'render, S = ()> {
    engine: &'render Engine<'render>,
    template: &'render Template<'render>,
    globals: Globals<'render, S>,
}

#[cfg(feature = "serde")]
enum Globals<'render, S = ()> {
    Serde(S),
    Borrowed(&'render Value),
    Fn(Box<ValueFn<'render>>),
}

#[cfg(feature = "serde")]
impl<'render, S> Renderer<'render, S> {
    pub(crate) fn with_serde(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: S,
    ) -> Self
    where
        S: ::serde::Serialize,
    {
        Self {
            engine,
            template,
            globals: Globals::Serde(globals),
        }
    }

    pub(crate) fn with_value(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: &'render Value,
    ) -> Self {
        Self {
            engine,
            template,
            globals: Globals::Borrowed(globals),
        }
    }

    pub(crate) fn with_value_fn(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        value_fn: Box<ValueFn<'render>>,
    ) -> Self {
        Self {
            engine,
            template,
            globals: Globals::Fn(value_fn),
        }
    }

    /// Render the template to a string.
    pub fn to_string(self) -> Result<String>
    where
        S: ::serde::Serialize,
    {
        match self.globals {
            Globals::Serde(ctx) => {
                let value = crate::to_value(ctx)?;
                let stack = Stack::new(&value);
                to_string(self.engine, self.template, stack)
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_string(self.engine, self.template, stack)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_string(self.engine, self.template, stack)
            }
        }
    }

    /// Render the template to the given writer.
    pub fn to_writer<W>(self, w: W) -> Result<()>
    where
        W: io::Write,
        S: ::serde::Serialize,
    {
        match self.globals {
            Globals::Serde(ctx) => {
                let value = crate::to_value(ctx)?;
                let stack = Stack::new(&value);
                to_writer(self.engine, self.template, stack, w)
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_writer(self.engine, self.template, stack, w)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_writer(self.engine, self.template, stack, w)
            }
        }
    }
}

/// A renderer that interprets a compiled [`Template`][crate::Template].
///
/// This struct is created by one of the following functions:
/// - [`Template{,Ref}::render`][crate::Template::render]
/// - [`Template{,Ref}::render_from`][crate::Template::render_from]
/// - [`Template{,Ref}::render_from_fn`][crate::Template::render_from_fn]
#[cfg(not(feature = "serde"))]
#[must_use = "must call `.to_string()` or `.to_writer(..)` on the renderer"]
pub struct Renderer<'render> {
    engine: &'render Engine<'render>,
    template: &'render Template<'render>,
    globals: Globals<'render>,
}

#[cfg(not(feature = "serde"))]
enum Globals<'render> {
    Borrowed(&'render Value),
    Fn(Box<ValueFn<'render>>),
}

#[cfg(not(feature = "serde"))]
impl<'render> Renderer<'render> {
    pub(crate) fn with_value(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: &'render Value,
    ) -> Self {
        Self {
            engine,
            template,
            globals: Globals::Borrowed(globals),
        }
    }

    pub(crate) fn with_value_fn(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        value_fn: Box<ValueFn<'render>>,
    ) -> Self {
        Self {
            engine,
            template,
            globals: Globals::Fn(value_fn),
        }
    }

    /// Render the template to a string.
    pub fn to_string(self) -> Result<String> {
        match self.globals {
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_string(self.engine, self.template, stack)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_string(self.engine, self.template, stack)
            }
        }
    }

    /// Render the template to the given writer.
    pub fn to_writer<W>(self, w: W) -> Result<()>
    where
        W: io::Write,
    {
        match self.globals {
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_writer(self.engine, self.template, stack, w)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_writer(self.engine, self.template, stack, w)
            }
        }
    }
}
