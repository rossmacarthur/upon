//! A tiny template engine.
//!
//! # Features
//!
//! - Configurable template delimiters: `<? value ?>`
//! - Rendering values: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} User is enabled {% endif %}`
//! - Customizable filter functions: `{{ value | my_filter }}`
//!
//! # Examples
//!
//! ### Render data constructed using the macro
//!
//! ```
//! use upon::{Engine, data};
//!
//! let result = Engine::new()
//!     .compile("Hello {{ value }}")?
//!     .render(data! { value: "World!" })?;
//!
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Render a template using custom tags
//!
//! ```
//! use upon::{data, Engine, Delimiters};
//!
//! let result = Engine::with_delims(Delimiters::new("<?", "?>", "<%", "%>"))
//!     .compile("Hello <? value ?>")?
//!     .render(data! { value: "World!" })?;
//!
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Render using structured data
//!
//! ```
//! use upon::Engine;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Data {
//!     user: User,
//! }
//! #[derive(Serialize)]
//! struct User {
//!     name: String,
//! }
//!
//! let data = Data { user: User { name: "John Smith".into() } };
//!
//! let result = Engine::new().compile("Hello {{ user.name }}")?.render(data)?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Named templates
//!
//! ```
//! use upon::{data, Engine};
//!
//! let mut engine = Engine::new();
//! engine.add_template("hello", "Hello {{ value }}")?;
//!
//! let result = engine.render("hello", data! { value: "World!" })?;
//!
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Transform data using filters
//!
//! ```
//! use upon::{data, Engine, Value};
//!
//! fn lower(mut v: Value) -> Value {
//!     if let Value::String(s) = &mut v {
//!         *s = s.to_lowercase();
//!     }
//!     v
//! }
//!
//! let mut engine = Engine::new();
//! engine.add_filter("lower", lower);
//!
//! let result = engine
//!     .compile("Hello {{ value | lower }}")?
//!     .render(data! { value: "WORLD!" })?;
//!
//! assert_eq!(result, "Hello world!");
//! # Ok::<(), upon::Error>(())
//! ```

mod ast;
mod compile;
mod error;
mod lex;
mod macros;
mod span;
pub mod value;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

pub use crate::error::{Error, Result};
use crate::span::Span;
pub use crate::value::{to_value, Value};

/// The compilation and rendering engine.
#[derive(Clone)]
pub struct Engine<'e> {
    delims: Delimiters<'e>,
    templates: HashMap<String, ast::Template<'e>>,
    filters: HashMap<String, Arc<dyn Fn(Value) -> Value + Send + Sync + 'e>>,
}

/// Delimiter configuration.
#[derive(Debug, Clone)]
pub struct Delimiters<'e> {
    begin_expr: &'e str,
    end_expr: &'e str,
    begin_block: &'e str,
    end_block: &'e str,
}

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template<'e> {
    engine: &'e Engine<'e>,
    template: ast::Template<'e>,
}

impl<'e> Default for Engine<'e> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'e> Engine<'e> {
    pub fn new() -> Self {
        Self {
            delims: Delimiters::default(),
            templates: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn with_delims(delims: Delimiters<'e>) -> Self {
        Self {
            delims,
            templates: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Value) -> Value + Send + Sync + 'e,
    {
        self.filters.insert(name.into(), Arc::new(f));
    }

    pub fn remove_filter<Q: ?Sized>(&mut self, name: &Q)
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.filters.remove(name);
    }

    /// Add a new named template to the engine.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::{data, Engine};
    ///
    /// let mut engine = Engine::new();
    /// engine.add_template("hello", "Hello {{ test }}!")?;
    /// # Ok::<(), upon::Error>(())
    /// ```
    pub fn add_template(&mut self, name: impl Into<String>, source: &'e str) -> Result<()> {
        let t = compile::template(source, &self.delims)?;
        self.templates.insert(name.into(), t);
        Ok(())
    }

    /// Remove a named template from the engine.
    ///
    /// # Panics
    ///
    /// If the template does not exist.
    #[track_caller]
    pub fn remove_template(&mut self, name: &'e str) {
        self.templates.remove(name).unwrap();
    }

    /// Compile an unamed template and return it.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::{data, Engine};
    ///
    /// let engine = Engine::new();
    ///
    ///  let result = engine
    ///     .compile("Hello {{ test }}!")?
    ///     .render(data! { test: "World" })?;
    ///
    /// assert_eq!(result, "Hello World!");
    /// # Ok::<(), upon::Error>(())
    /// ```
    pub fn compile(&'e self, source: &'e str) -> Result<Template<'e>> {
        Template::compile(self, source)
    }

    /// Render a named template to a string using the provided data.
    ///
    /// # Panics
    ///
    /// If the template does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::{data, Engine};
    ///
    /// let mut engine = Engine::new();
    /// engine.add_template("hello", "Hello {{ test }}!")?;
    ///
    /// let result = engine.render("hello", data! { test: "World" })?;
    ///
    /// assert_eq!(result, "Hello World!");
    /// # Ok::<(), upon::Error>(())
    /// ```
    pub fn render<S>(&'e self, _name: &str, _data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        todo!()
    }
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filters = f
            .debug_map()
            .entries(self.filters.keys().map(|k| (k, "<filter>")))
            .finish();
        f.debug_struct("Engine")
            .field("delims", &self.delims)
            .field("templates", &self.templates)
            .field("filters", &filters)
            .finish()
    }
}

impl Default for Delimiters<'_> {
    fn default() -> Self {
        Self::new("{{", "}}", "{%", "%}")
    }
}

impl<'e> Delimiters<'e> {
    /// Returns a new tag configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::Delimiters;
    ///
    /// // Like Liquid / Jinja, this is the same as `Delimiters::default()`
    /// let delims = Delimiters::new("{{", "}}", "{%", "%}");
    ///
    /// // Use single braces for expressions and double braces for blocks
    /// let delims = Delimiters::new("{", "}", "{{", "}}");
    ///
    /// // Completely custom!
    /// let delims = Delimiters::new("<?", "?>", "<%", "%>");
    /// ```
    pub const fn new(
        begin_expr: &'e str,
        end_expr: &'e str,
        begin_block: &'e str,
        end_block: &'e str,
    ) -> Self {
        Self {
            begin_expr,
            end_expr,
            begin_block,
            end_block,
        }
    }
}

impl<'e> Template<'e> {
    #[inline]
    fn compile(engine: &'e Engine<'e>, source: &'e str) -> Result<Self> {
        let template = compile::template(source, &engine.delims)?;
        Ok(Self { engine, template })
    }

    #[inline]
    pub fn source(&self) -> &'e str {
        self.template.source
    }

    /// Render the template to a string using the provided data.
    #[inline]
    pub fn render<S>(&self, _data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        todo!()
    }
}
