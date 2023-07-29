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
    settings: &RenderSettings,
) -> Result<String> {
    let mut s = String::with_capacity(template.source.len());
    let mut f = Formatter::with_string(&mut s);
    RendererImpl {
        engine,
        template,
        stack,
    }
    .render(&mut f, settings)?;
    Ok(s)
}

fn to_writer<'a, W>(
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
    stack: Stack<'a>,
    writer: W,
    settings: &RenderSettings,
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
    .render(&mut f, settings)
    .map_err(|err| w.take_err().map(Error::from).unwrap_or(err))
}

/// A renderer that interprets a compiled [`Template`][crate::Template].
///
/// This struct is created by one of the following functions:
/// - [`Template{,Ref}::render`][crate::Template::render]
/// - [`Template{,Ref}::render_from`][crate::Template::render_from]
/// - [`Template{,Ref}::render_from_fn`][crate::Template::render_from_fn]
#[must_use = "must call `.to_string()` or `.to_writer(..)` on the renderer"]
pub struct Renderer<'render> {
    engine: &'render Engine<'render>,
    template: &'render Template<'render>,
    globals: Globals<'render>,
    max_include_depth: Option<usize>,
}

enum Globals<'render> {
    Owned(Result<Value>),
    Borrowed(&'render Value),
    Fn(Box<ValueFn<'render>>),
}

pub(crate) struct RenderSettings {
    max_include_depth: usize,
}

impl<'render> Renderer<'render> {
    fn new(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: Globals<'render>,
    ) -> Self {
        Self {
            engine,
            template,
            globals,
            max_include_depth: None,
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn with_serde<S>(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: S,
    ) -> Self
    where
        S: ::serde::Serialize,
    {
        Self::new(engine, template, Globals::Owned(crate::to_value(globals)))
    }

    pub(crate) fn with_value(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        globals: &'render Value,
    ) -> Self {
        Self::new(engine, template, Globals::Borrowed(globals))
    }

    pub(crate) fn with_value_fn(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        value_fn: Box<ValueFn<'render>>,
    ) -> Self {
        Self::new(engine, template, Globals::Fn(value_fn))
    }

    /// Set the maximum length of the template render stack.
    ///
    /// This is the maximum number of nested `{% include ... %}` statements that
    /// are allowed during rendering, as counted from the root template.
    ///
    /// Defaults to the engine setting.
    pub fn with_max_include_depth(mut self, depth: usize) -> Self {
        self.max_include_depth = Some(depth);
        self
    }

    /// Render the template to a string.
    pub fn to_string(self) -> Result<String> {
        let settings = get_settings(&self);
        match self.globals {
            Globals::Owned(result) => {
                let value = result?;
                let stack = Stack::new(&value);
                to_string(self.engine, self.template, stack, &settings)
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_string(self.engine, self.template, stack, &settings)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_string(self.engine, self.template, stack, &settings)
            }
        }
    }

    /// Render the template to the given writer.
    pub fn to_writer<W>(self, w: W) -> Result<()>
    where
        W: io::Write,
    {
        let settings = get_settings(&self);
        match self.globals {
            Globals::Owned(result) => {
                let value = result?;
                let stack = Stack::new(&value);
                to_writer(self.engine, self.template, stack, w, &settings)
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_writer(self.engine, self.template, stack, w, &settings)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_writer(self.engine, self.template, stack, w, &settings)
            }
        }
    }
}

fn get_settings(renderer: &Renderer) -> RenderSettings {
    RenderSettings {
        max_include_depth: renderer
            .max_include_depth
            .unwrap_or(renderer.engine.max_include_depth),
    }
}
