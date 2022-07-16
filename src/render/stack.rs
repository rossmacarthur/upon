use crate::render::iter::LoopState;
use crate::render::value::{lookup_path, lookup_path_maybe};
use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result};

#[cfg_attr(test, derive(Debug))]
pub struct Stack<'source, 'render> {
    stack: Vec<State<'source, 'render>>,
}

#[cfg_attr(test, derive(Debug))]
pub enum State<'source, 'render> {
    /// An entire scope of variables, always a map
    Scope(ValueCow<'render>),

    /// A single variable.
    Var(&'source ast::Ident<'source>, ValueCow<'render>),

    /// The current state of a loop iteration
    Loop(LoopState<'source, 'render>),

    /// Used to represent a template boundary.
    Boundary,
}

impl<'source, 'render> Stack<'source, 'render> {
    pub fn new(globals: ValueCow<'render>) -> Self {
        Self {
            stack: vec![State::Scope(globals)],
        }
    }

    /// Resolves a path to a variable on the stack.
    pub fn lookup_path(
        &self,
        source: &str,
        path: &[ast::Ident<'source>],
    ) -> Result<ValueCow<'render>> {
        for state in self.stack.iter().rev() {
            match state {
                State::Scope(scope) => match lookup_path_maybe(source, scope, path)? {
                    Some(value) => return Ok(value),
                    None => continue,
                },

                State::Var(name, var) if path[0].raw == name.raw => {
                    return lookup_path(source, var, &path[1..]);
                }

                State::Loop(loop_state) => {
                    if let Some(value) = loop_state.lookup_path(source, path)? {
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
        Err(Error::new("not found in this scope", source, path[0].span))
    }

    pub fn push(&mut self, state: State<'source, 'render>) {
        self.stack.push(state);
    }

    pub fn last_loop_state_mut(&mut self) -> &mut LoopState<'source, 'render> {
        match self.stack.last_mut().unwrap() {
            State::Loop(loop_state) => loop_state,
            _ => panic!("expected loop state"),
        }
    }

    pub fn pop_var(&mut self) -> (&'source ast::Ident<'source>, ValueCow<'render>) {
        match self.stack.pop().unwrap() {
            State::Var(name, value) => (name, value),
            _ => panic!("expected variable"),
        }
    }

    pub fn pop_loop_state(&mut self) -> LoopState<'source, 'render> {
        match self.stack.pop().unwrap() {
            State::Loop(state) => state,
            _ => panic!("expected loop state"),
        }
    }
}
