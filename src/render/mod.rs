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

fn to_string(inner: RendererInner<'_>, stack: Stack<'_>) -> Result<String> {
    let mut s = String::with_capacity(inner.template.source.len());
    let mut f = Formatter::with_string(&mut s);
    RendererImpl { inner, stack }.render(&mut f)?;
    Ok(s)
}

fn to_writer<W>(inner: RendererInner<'_>, stack: Stack<'_>, writer: W) -> Result<()>
where
    W: io::Write,
{
    let mut w = Writer::new(writer);
    let mut f = Formatter::with_writer(&mut w);
    RendererImpl { inner, stack }
        .render(&mut f)
        .map_err(|err| w.take_err().map(Error::from).unwrap_or(err))
}

type TemplateFn<'a> = dyn FnMut(&str) -> std::result::Result<&'a crate::Template<'a>, String> + 'a;

/// A renderer that interprets a compiled [`Template`][crate::Template] or
/// [`TemplateRef`][crate::TemplateRef].
///
/// This struct is created by one of the following functions:
/// - [`Template{,Ref}::render`][crate::Template::render]
/// - [`Template{,Ref}::render_from`][crate::Template::render_from]
/// - [`Template{,Ref}::render_from_fn`][crate::Template::render_from_fn]
#[must_use = "must call `.to_string()` or `.to_writer(..)` on the renderer"]
pub struct Renderer<'render> {
    globals: Globals<'render>,
    inner: RendererInner<'render>,
}

enum Globals<'render> {
    Owned(Result<Value>),
    Borrowed(&'render Value),
    Fn(Box<ValueFn<'render>>),
}
pub(crate) struct RendererInner<'render> {
    engine: &'render Engine<'render>,
    template: &'render Template<'render>,
    template_name: Option<&'render str>,
    max_include_depth: Option<usize>,
    template_fn: Option<Box<TemplateFn<'render>>>,
}

#[cfg(internal_debug)]
impl std::fmt::Debug for RendererInner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererInner")
            .field("engine", &self.engine)
            .field("template", &self.template)
            .field("max_include_depth", &self.max_include_depth)
            .finish_non_exhaustive()
    }
}

impl<'render> Renderer<'render> {
    fn new(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        template_name: Option<&'render str>,
        globals: Globals<'render>,
    ) -> Self {
        Self {
            globals,
            inner: RendererInner {
                engine,
                template,
                template_name,
                max_include_depth: None,
                template_fn: None,
            },
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn with_serde<S>(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        template_name: Option<&'render str>,
        globals: S,
    ) -> Self
    where
        S: ::serde::Serialize,
    {
        Self::new(
            engine,
            template,
            template_name,
            Globals::Owned(crate::to_value(globals)),
        )
    }

    pub(crate) fn with_value(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        template_name: Option<&'render str>,
        globals: &'render Value,
    ) -> Self {
        Self::new(engine, template, template_name, Globals::Borrowed(globals))
    }

    pub(crate) fn with_value_fn(
        engine: &'render Engine<'render>,
        template: &'render Template<'render>,
        template_name: Option<&'render str>,
        value_fn: Box<ValueFn<'render>>,
    ) -> Self {
        Self::new(engine, template, template_name, Globals::Fn(value_fn))
    }

    /// Set a function that is called when a template is included.
    ///
    /// This allows custom template resolution on a per render basis. The
    /// default is to look for the template with the exact matching name in the
    /// engine, i.e. the same as
    /// [`Engine::get_template`][crate::Engine::get_template].
    pub fn with_template_fn<F>(mut self, template_fn: F) -> Self
    where
        F: FnMut(&str) -> std::result::Result<&'render crate::Template<'render>, String> + 'render,
    {
        self.inner.template_fn = Some(Box::new(template_fn));
        self
    }

    /// Set the maximum length of the template render stack.
    ///
    /// This is the maximum number of nested `{% include ... %}` statements that
    /// are allowed during rendering, as counted from the root template.
    ///
    /// Defaults to the engine setting.
    pub fn with_max_include_depth(mut self, depth: usize) -> Self {
        self.inner.max_include_depth = Some(depth);
        self
    }

    /// Render the template to a string.
    pub fn to_string(self) -> Result<String> {
        let Self { globals, inner } = self;
        match globals {
            Globals::Owned(result) => {
                let value = result?;
                let stack = Stack::new(&value);
                let x = to_string(inner, stack);
                drop(value);
                x
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_string(inner, stack)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_string(inner, stack)
            }
        }
    }

    /// Render the template to the given writer.
    pub fn to_writer<W>(self, w: W) -> Result<()>
    where
        W: io::Write,
    {
        let Self { globals, inner } = self;
        match globals {
            Globals::Owned(result) => {
                let value = result?;
                let stack = Stack::new(&value);
                to_writer(inner, stack, w)
            }
            Globals::Borrowed(value) => {
                let stack = Stack::new(value);
                to_writer(inner, stack, w)
            }
            Globals::Fn(value_fn) => {
                let stack = Stack::with_value_fn(&value_fn);
                to_writer(inner, stack, w)
            }
        }
    }
}
