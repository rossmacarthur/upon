use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

use crate::{Result, Template, Value};

/// The compilation and rendering engine.
#[derive(Clone)]
pub struct Engine<'e> {
    pub(crate) begin_tag: &'e str,
    pub(crate) end_tag: &'e str,
    pub(crate) filters: HashMap<String, Arc<dyn Fn(Value) -> Value + 'e>>,
}

impl Default for Engine<'_> {
    fn default() -> Self {
        Self {
            begin_tag: "{{",
            end_tag: "}}",
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
            filters: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Value) -> Value + 'e,
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

    pub fn compile(&'e self, tmpl: &'e str) -> Result<Template<'e>> {
        Template::with_env(tmpl, self)
    }

    /// Render the template to a string using the provided data.
    pub fn render<S>(&'e self, tmpl: &str, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        self.compile(tmpl)?.render(data)
    }
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filters = f
            .debug_map()
            .entries(self.filters.keys().map(|k| (k, "<filter>")))
            .finish();
        f.debug_struct("Env")
            .field("begin_tag", &self.begin_tag)
            .field("end_tag", &self.end_tag)
            .field("filters", &filters)
            .finish()
    }
}
