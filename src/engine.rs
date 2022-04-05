use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

use crate::{RawTemplate, Result, Template, Value};

/// The compilation and rendering engine.
#[derive(Clone)]
pub struct Engine<'e> {
    pub(crate) begin_tag: &'e str,
    pub(crate) end_tag: &'e str,
    pub(crate) templates: HashMap<&'e str, RawTemplate<'e>>,
    pub(crate) filters: HashMap<String, Arc<dyn Fn(Value) -> Value + Send + Sync + 'e>>,
}

impl Default for Engine<'_> {
    fn default() -> Self {
        Self {
            begin_tag: "{{",
            end_tag: "}}",
            templates: HashMap::new(),
            filters: HashMap::new(),
        }
    }
}

impl<'e> Engine<'e> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tags(begin_tag: &'e str, end_tag: &'e str) -> Self {
        Self {
            begin_tag,
            end_tag,
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
    pub fn add_template(&mut self, name: &'e str, source: &'e str) -> Result<()> {
        let t = RawTemplate::compile(self, source)?;
        self.templates.insert(name, t);
        Ok(())
    }

    /// Remove a named template from the engine.
    ///
    /// # Panics
    ///
    /// If the template does not exist.
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
    pub fn render<S>(&'e self, name: &str, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        self.templates.get(name).unwrap().render(self, data)
    }
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filters = f
            .debug_map()
            .entries(self.filters.keys().map(|k| (k, "<filter>")))
            .finish();
        f.debug_struct("Engine")
            .field("begin_tag", &self.begin_tag)
            .field("end_tag", &self.end_tag)
            .field("templates", &self.templates)
            .field("filters", &filters)
            .finish()
    }
}
