use crate::render::iter::LoopState;
use crate::render::value::{lookup_path, lookup_path_maybe};
use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Stack<'a> {
    stack: Vec<State<'a>>,
}

#[cfg_attr(internal_debug, derive(Debug))]
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

impl<'a> Stack<'a> {
    pub fn new(globals: ValueCow<'a>) -> Self {
        Self {
            stack: vec![State::Scope(globals)],
        }
    }

    /// Resolves a path to a variable on the stack.
    pub fn lookup_var(&self, source: &str, v: &ast::Var) -> Result<ValueCow<'a>> {
        for state in self.stack.iter().rev() {
            match state {
                State::Scope(scope) => match lookup_path_maybe(source, scope, v)? {
                    Some(value) => return Ok(value),
                    None => continue,
                },

                State::Var(name, var) if source[v.first().span()] == source[name.span] => {
                    return lookup_path(source, var, v.rest());
                }

                State::Loop(loop_state) => {
                    if let Some(value) = loop_state.lookup_var(source, v)? {
                        return Ok(value);
                    }
                }

                State::Boundary => {
                    // We've reached the template boundary stop searching
                    break;
                }

                _ => {}
            }
        }
        Err(Error::render(
            "not found in this scope",
            source,
            v.first().span(),
        ))
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
