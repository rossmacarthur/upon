//! A tiny template engine.
//!
//! # Features
//!
//! - Rendering values: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} ... {% endif %}`
//! - Loops: `{% for user in users %} ... {% endfor %}`
//! - Customizable filter functions: `{{ value | my_filter }}`
//! - Configurable template delimiters: `<? user.name ?>`, `(( if user.enabled ))`
//! - Supports any [`serde`][serde] serializable values.
//! - Macro for quick rendering: `data!{ name: "John", age: 42 }`
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
//! let result = template.render(upon::data!{ user: { name: "John Smith" }})?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
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
//! fn lower(mut v: upon::Value) -> upon::Value {
//!     if let upon::Value::String(s) = &mut v {
//!         *s = s.to_lowercase();
//!     }
//!     v
//! }
//!
//! let mut engine = upon::Engine::new();
//! engine.add_filter("lower", lower);
//!
//! let result = engine
//!     .compile("Hello {{ value | lower }}")?
//!     .render(upon::data! { value: "WORLD!" })?;
//!
//! assert_eq!(result, "Hello world!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Render a template using custom tags
//!
//! ```
//! let result = upon::Engine::with_delims("<?", "?>", "<%", "%>")
//!     .compile("Hello <? user.name ?>")?
//!     .render(upon::data!{ user: { name: "John Smith" }})?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```

mod ast;
mod compile;
mod error;
mod lex;
mod macros;
mod render;
mod span;
pub mod value;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub use crate::error::{Error, Result};
use crate::span::Span;
pub use crate::value::{to_value, Value};

/// The compilation and rendering engine.
#[derive(Clone)]
pub struct Engine<'engine> {
    begin_expr: &'engine str,
    end_expr: &'engine str,
    begin_block: &'engine str,
    end_block: &'engine str,
    filters: HashMap<String, Arc<dyn Fn(Value) -> Value + Send + Sync + 'engine>>,
}

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: ast::Template<'source>,
}

impl Default for Engine<'_> {
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
            .finish()
    }
}

impl<'engine> Engine<'engine> {
    /// Construct a new engine.
    pub fn new() -> Self {
        Self {
            begin_expr: "{{",
            end_expr: "}}",
            begin_block: "{%",
            end_block: "%}",
            filters: HashMap::new(),
        }
    }

    /// Construct a new engine with custom delimiters.
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
        }
    }

    /// Add a new filter to the engine.
    pub fn add_filter<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Value) -> Value + Send + Sync + 'engine,
    {
        self.filters.insert(name.into(), Arc::new(f));
    }

    /// Compile a template.
    pub fn compile<'source>(&self, source: &'source str) -> Result<Template<'_, 'source>> {
        let template = compile::Parser::new(self, source).parse_template()?;
        Ok(Template {
            engine: self,
            template,
        })
    }
}

impl<'engine, 'source> Template<'engine, 'source> {
    pub fn source(&self) -> &'source str {
        self.template.source
    }

    /// Render the template to a string using the provided data.
    #[inline]
    pub fn render<S>(&self, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::Renderer::new(&self.engine, &self.template).render(to_value(data)?)
    }
}
