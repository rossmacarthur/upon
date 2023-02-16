use crate::render::iter::LoopState;
use crate::render::value::{lookup_path, lookup_path_maybe};
use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result, ValueFn, ValueKey};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Stack<'a> {
    /// A function for fetching values.
    value_fn: Option<&'a ValueFn<'a>>,
    /// The variable stack.
    stack: Vec<State<'a>>,
}

pub enum State<'a> {
    /// An entire scope of variables, always a map
    Scope(ValueCow<'a>),

    /// A single variable.
    Var(&'a ast::Ident, ValueCow<'a>),

    /// The current state of a loop iteration
    Loop(LoopState<'a>),

    /// Used to represent a template boundary.
    Boundary,
}

#[cfg(internal_debug)]
impl std::fmt::Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope(scope) => f.debug_tuple("Scope").field(scope).finish(),
            Self::Var(ident, value) => f.debug_tuple("Var").field(ident).field(value).finish(),
            Self::Loop(state) => f.debug_tuple("Loop").field(state).finish(),
            Self::Boundary => write!(f, "Boundary"),
        }
    }
}

impl<'a> Stack<'a> {
    pub fn new(globals: ValueCow<'a>) -> Self {
        Self {
            value_fn: None,
            stack: vec![State::Scope(globals)],
        }
    }

    pub fn with_value_fn(f: &'a ValueFn<'a>) -> Self {
        Self {
            value_fn: Some(f),
            stack: vec![],
        }
    }

    /// Resolves a path to a variable on the stack, falling back to a value fn
    /// if it is not found.
    pub fn lookup_var_or_call_value_fn(&self, source: &str, v: &ast::Var) -> Result<ValueCow<'a>> {
        if let Some(v) = self.lookup_var(source, &v)? {
            return Ok(v);
        }

        if let Some(f) = self.value_fn {
            let path: Vec<_> = v
                .path
                .iter()
                .map(|key| match key {
                    ast::Key::List(k) => ValueKey::List(k.value),
                    ast::Key::Map(k) => ValueKey::Map(&source[k.span]),
                })
                .collect();
            return f(&path)
                .map(ValueCow::Owned)
                .map_err(|reason| Error::render(reason, source, v.span()));
        }

        Err(Error::render(
            "not found in this scope",
            source,
            v.first().span(),
        ))
    }

    /// Resolves a path to a variable on the stack.
    pub fn lookup_var(&self, source: &str, v: &ast::Var) -> Result<Option<ValueCow<'a>>> {
        for state in self.stack.iter().rev() {
            match state {
                State::Scope(scope) => match lookup_path_maybe(source, scope, v)? {
                    Some(value) => return Ok(Some(value)),
                    None => continue,
                },

                State::Var(name, var) if source[v.first().span()] == source[name.span] => {
                    return lookup_path(source, var, v.rest()).map(Some);
                }

                State::Loop(loop_state) => {
                    if let Some(value) = loop_state.lookup_var(source, v)? {
                        return Ok(Some(value));
                    }
                }

                State::Boundary => {
                    // We've reached the template boundary stop searching
                    break;
                }

                _ => {}
            }
        }

        Ok(None)
    }

    pub fn push(&mut self, state: State<'a>) {
        self.stack.push(state);
    }

    pub fn last_loop_state_mut(&mut self) -> &mut LoopState<'a> {
        match self.stack.last_mut().unwrap() {
            State::Loop(loop_state) => loop_state,
            _ => panic!("expected loop state"),
        }
    }

    pub fn pop_scope(&mut self) -> ValueCow<'a> {
        match self.stack.pop().unwrap() {
            State::Scope(globals) => globals,
            _ => panic!("expected scope"),
        }
    }

    pub fn pop_var(&mut self) -> (&'a ast::Ident, ValueCow<'a>) {
        match self.stack.pop().unwrap() {
            State::Var(name, value) => (name, value),
            _ => panic!("expected variable"),
        }
    }

    pub fn pop_loop_state(&mut self) -> LoopState<'a> {
        match self.stack.pop().unwrap() {
            State::Loop(state) => state,
            _ => panic!("expected loop state"),
        }
    }

    pub fn pop_boundary(&mut self) {
        match self.stack.pop().unwrap() {
            State::Boundary => {}
            _ => panic!("expected boundary"),
        }
    }
}
