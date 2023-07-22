use crate::render::iter::LoopState;
use crate::render::value::{lookup_path, lookup_path_maybe};
use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result, Value, ValueAccess, ValueAccessOp, ValueFn, ValueMember};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Stack<'a> {
    stack: Vec<State<'a>>,
}

pub enum State<'a> {
    /// A function for fetching values.
    ValueFn(&'a ValueFn<'a>),

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
            Self::ValueFn(_) => f.debug_tuple("ValueFn").field(&(..)).finish(),
            Self::Scope(scope) => f.debug_tuple("Scope").field(scope).finish(),
            Self::Var(ident, value) => f.debug_tuple("Var").field(ident).field(value).finish(),
            Self::Loop(state) => f.debug_tuple("Loop").field(state).finish(),
            Self::Boundary => write!(f, "Boundary"),
        }
    }
}

impl<'a> Stack<'a> {
    pub fn new(globals: &'a Value) -> Self {
        Self {
            stack: vec![State::Scope(ValueCow::Borrowed(globals))],
        }
    }

    pub fn with_value_fn(f: &'a ValueFn<'a>) -> Self {
        Self {
            stack: vec![State::ValueFn(f)],
        }
    }

    /// Resolves a path to a variable on the stack.
    pub fn lookup_var(&self, source: &str, v: &ast::Var) -> Result<ValueCow<'a>> {
        for state in self.stack.iter().rev() {
            match state {
                State::ValueFn(value_fn) => {
                    let path: Vec<_> = v
                        .path
                        .iter()
                        .map(|member| {
                            let op = match member.op {
                                ast::AccessOp::Direct => ValueAccessOp::Direct,
                                ast::AccessOp::Optional => ValueAccessOp::Optional,
                            };
                            let access = match member.access {
                                ast::Access::Index(a) => ValueAccess::Index(a.value),
                                ast::Access::Key(a) => ValueAccess::Key(&source[a.span]),
                            };
                            ValueMember { op, access }
                        })
                        .collect();
                    return value_fn(&path)
                        .map(ValueCow::Owned)
                        .map_err(|reason| Error::render(reason, source, v.span()));
                }

                State::Scope(scope) => match lookup_path_maybe(source, scope, &v.path)? {
                    Some(value) => return Ok(value),
                    None => continue,
                },

                State::Var(name, var) if source[v.first().access.span()] == source[name.span] => {
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
            v.first().span,
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
