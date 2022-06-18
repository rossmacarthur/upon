//! A simple, powerful template engine.
//!
//! # Features
//!
//! - Expressions: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} ... {% endif %}`
//! - Loops: `{% for user in users %} ... {% endfor %}`
//! - Customizable filter functions: `{{ user.name | lower }}`
//! - Configurable template syntax: `<? user.name ?>`, `(( if user.enabled ))`
//! - Render using any [`serde`][serde] serializable values.
//! - Render using a quick context with a convenient macro:
//!   `upon::value!{ name: "John", age: 42 }`
//! - Minimal dependencies.
//!
//! # Introduction
//!
//! Your entry point is the compilation and rendering [`Engine`], this stores
//! the syntax config and filter functions. Generally, you only need to
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
//! The template can then be rendered by calling [`.render()`][TemplateRef::render].
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
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//!
//! // later...
//!
//! let template = engine.get_template("hello").unwrap();
//! let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
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
//! struct Context { user: User }
//!
//! #[derive(serde::Serialize)]
//! struct User { name: String }
//!
//! let ctx = Context { user: User { name: "John Smith".into() } };
//!
//! let result = upon::Engine::new()
//!     .compile("Hello {{ user.name }}")?
//!     .render(&ctx)?;
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
//! let mut engine = upon::Engine::new();
//!
//! engine.add_filter("lower", |v| {
//!     if let upon::Value::String(s) = v {
//!         *s = s.to_lowercase();
//!     }
//! });
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
//! The template syntax can be set by constructing an engine using
//! [`Engine::with_syntax`].
//!
//! ```
//! let syntax = upon::Syntax::builder().expr("<?", "?>").block("<%", "%>").build();
//!
//! let result = upon::Engine::with_syntax(syntax)
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

pub use crate::error::Error;
pub use crate::types::syntax::{Syntax, SyntaxBuilder};
pub use crate::value::{to_value, Value};

use crate::compile::Searcher;
use crate::types::program;

/// A type alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The compilation and rendering engine.
pub struct Engine<'engine> {
    searcher: Searcher,
    filters: HashMap<&'engine str, FilterFn>,
    templates: HashMap<&'engine str, program::Template<'engine>>,
}

/// A filter function or closure.
type FilterFn = Arc<dyn Fn(&mut Value) + Sync + Send + 'static>;

/// A compiled template.
#[cfg_attr(test, derive(Debug))]
pub struct Template<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: program::Template<'source>,
}

/// A reference to a compiled template in an [`Engine`].
#[cfg_attr(test, derive(Debug))]
pub struct TemplateRef<'engine> {
    engine: &'engine Engine<'engine>,
    template: &'engine program::Template<'engine>,
}

impl<'engine> Default for Engine<'engine> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'engine> Engine<'engine> {
    /// Construct a new engine.
    #[inline]
    pub fn new() -> Engine<'engine> {
        Self {
            searcher: Searcher::new(Syntax::default()),
            filters: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    /// Construct a new engine with custom syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::{Engine, Syntax};
    ///
    /// let syntax = Syntax::builder().expr("<{", "}>").block("<[", "]>").build();
    /// let engine = Engine::with_syntax(syntax);
    /// ```
    #[inline]
    pub fn with_syntax(syntax: Syntax<'engine>) -> Self {
        Self {
            searcher: Searcher::new(syntax),
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

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Engine");
        d.field("searcher", &self.searcher);
        d.field("filters", &self.filters.keys());
        #[cfg(not(test))]
        {
            d.field("templates", &self.templates.keys()).finish()
        }
        #[cfg(test)]
        {
            d.field("templates", &self.templates).finish()
        }
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

#[cfg(not(test))]
impl fmt::Debug for Template<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Template")
            .field("engine", &self.engine)
            .finish_non_exhaustive()
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

#[cfg(not(test))]
impl fmt::Debug for TemplateRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TemplateRef")
            .field("engine", &self.engine)
            .finish_non_exhaustive()
    }
}
