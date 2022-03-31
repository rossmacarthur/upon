use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

use crate::{Result, Template, Value};

#[derive(Clone)]
pub struct Env<'env> {
    pub(crate) begin_tag: &'env str,
    pub(crate) end_tag: &'env str,
    pub(crate) filters: HashMap<String, Arc<dyn Fn(Value) -> Value + 'env>>,
}

impl<'env> Env<'env> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tags(begin_tag: &'env str, end_tag: &'env str) -> Self {
        Self {
            begin_tag,
            end_tag,
            filters: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Value) -> Value + 'env,
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

    pub fn compile(&'env self, tmpl: &'env str) -> Result<Template<'env>> {
        Template::with_env(tmpl, &self)
    }
}

impl fmt::Debug for Env<'_> {
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

impl Default for Env<'_> {
    fn default() -> Self {
        Self {
            begin_tag: "{{",
            end_tag: "}}",
            filters: HashMap::new(),
        }
    }
}
