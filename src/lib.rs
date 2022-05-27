//! A tiny template engine.
//!
//! # Features
//!
//! - Expressions: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} ... {% endif %}`
//! - Loops: `{% for user in users %} ... {% endfor %}`
//! - Customizable filter functions: `{{ user.name | lower }}`
//! - Configurable template delimiters: `<? user.name ?>`, `(( if user.enabled ))`
//! - Supports any [`serde`][serde] serializable values.
//! - Macro for quick rendering: `value!{ name: "John", age: 42 }`
//!
//! # Introduction
//!
//! Your entry point is the compilation and rendering [`Engine`], this stores
//! the delimiter settings and filter functions. Generally, you only need to
//! construct one engine.
//!
//! ```
//! let engine = upon::Engine::new();
//! ```
//!
//! Compiling a template returns a reference to it bound to the lifetime of the
//! engine and the template source.
//!
//! ```
//! # let engine = upon::Engine::new();
//! let template = engine.compile("Hello {{ user.name }}!")?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! The template can then be rendered by calling `.render()`.
//!
//! ```
//! # let engine = upon::Engine::new();
//! # let template = engine.compile("Hello {{ user.name }}!")?;
//! let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! You can also use [`add_template(name, ...)`][Engine::add_template] and
//! [`get_template(name).render(...)`][Engine::get_template] to store a template
//! by name in the engine.
//!
//! # Examples
//!
//! ### Render using structured data
//!
//! Here is the same example as above except using derived data.
//!
//! ```
//! #[derive(serde::Serialize)]
//! struct Data { user: User }
//! #[derive(serde::Serialize)]
//! struct User { name: String }
//!
//! let engine = upon::Engine::new();
//! let data = Data { user: User { name: "John Smith".into() } };
//! let template = engine.compile("Hello {{ user.name }}")?;
//! let result = template.render(&data)?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Transform data using filters
//!
//! Data can be transformed using registered filters.
//!
//! ```
//! // If the value is a string, make it lowercase
//! fn lower(v: &mut upon::Value) {
//!     if let upon::Value::String(s) = v {
//!         *s = s.to_lowercase();
//!     }
//! }
//!
//! let mut engine = upon::Engine::new();
//! engine.add_filter("lower", lower);
//!
//! let result = engine
//!     .compile("Hello {{ value | lower }}")?
//!     .render(upon::value! { value: "WORLD!" })?;
//!
//! assert_eq!(result, "Hello world!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Render a template using custom syntax
//!
//! ```
//! let result = upon::Engine::with_delims("<?", "?>", "<%", "%>")
//!     .compile("Hello <? user.name ?>")?
//!     .render(upon::value!{ user: { name: "John Smith" }})?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```

mod compile;
mod error;
mod macros;
mod render;
mod types;
pub mod value;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub use crate::error::{Error, Result};
use crate::types::prog;
pub use crate::value::{to_value, Value};

/// The compilation and rendering engine.
pub struct Engine<'engine> {
    begin_expr: &'engine str,
    end_expr: &'engine str,
    begin_block: &'engine str,
    end_block: &'engine str,
    filters: HashMap<&'engine str, Arc<dyn Fn(&mut Value) + Sync + Send + 'static>>,
    templates: HashMap<&'engine str, prog::Template<'engine>>,
}

/// A compiled template.
#[derive(Debug)]
pub struct Template<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: prog::Template<'source>,
}

/// A reference to a compiled template in an [`Engine`].
#[derive(Debug)]
pub struct TemplateRef<'engine> {
    engine: &'engine Engine<'engine>,
    template: &'engine prog::Template<'engine>,
}

impl<'engine> Default for Engine<'engine> {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filters = f
            .debug_map()
            .entries(self.filters.keys().map(|k| (k, "<filter>")))
            .finish();
        f.debug_struct("Engine")
            .field("begin_expr", &self.begin_expr)
            .field("end_expr", &self.end_expr)
            .field("begin_block", &self.begin_block)
            .field("end_block", &self.end_block)
            .field("filters", &filters)
            .field("templates", &self.templates)
            .finish()
    }
}

impl<'engine> Engine<'engine> {
    /// Construct a new engine.
    #[inline]
    pub fn new() -> Engine<'engine> {
        Self {
            begin_expr: "{{",
            end_expr: "}}",
            begin_block: "{%",
            end_block: "%}",
            filters: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    /// Construct a new engine with custom delimiters.
    #[inline]
    pub fn with_delims(
        begin_expr: &'engine str,
        end_expr: &'engine str,
        begin_block: &'engine str,
        end_block: &'engine str,
    ) -> Self {
        Self {
            begin_expr,
            end_expr,
            begin_block,
            end_block,
            filters: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    /// Add a new filter to the engine.
    #[inline]
    pub fn add_filter<F>(&mut self, name: &'engine str, f: F)
    where
        F: Fn(&mut Value) + Send + Sync + 'static,
    {
        self.filters.insert(name, Arc::new(f));
    }

    /// Compile a template.
    #[inline]
    pub fn compile<'source>(&self, source: &'source str) -> Result<Template<'_, 'source>> {
        let template = compile::template(self, source)?;
        Ok(Template {
            engine: self,
            template,
        })
    }

    /// Compile a template and store it with the given name.
    ///
    /// When using this function over [`.compile(..)`][Engine::compile] the
    /// template source lifetime needs to be as least as long as the engine
    /// lifetime.
    #[inline]
    pub fn add_template(&mut self, name: &'engine str, source: &'engine str) -> Result<()> {
        let template = compile::template(self, source)?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Lookup a template by name.
    #[inline]
    pub fn get_template(&self, name: &str) -> Option<TemplateRef<'_>> {
        self.templates.get(name).map(|template| TemplateRef {
            engine: self,
            template,
        })
    }
}

impl<'engine, 'source> Template<'engine, 'source> {
    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &'source str {
        self.template.source
    }

    /// Render the template to a string using the provided value.
    #[inline]
    pub fn render<S>(&self, s: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::template(self.engine, &self.template, to_value(s)?)
    }
}

impl<'engine> TemplateRef<'engine> {
    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &'engine str {
        self.template.source
    }

    /// Render the template to a string using the provided value.
    #[inline]
    pub fn render<S>(&self, s: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::template(self.engine, self.template, to_value(s)?)
    }
}
