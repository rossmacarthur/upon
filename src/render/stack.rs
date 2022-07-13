use crate::render::iter::LoopState;
use crate::render::value::index;
use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

#[cfg_attr(test, derive(Debug))]
pub struct Stack<'source, 'render> {
    source: &'source str,
    stack: Vec<State<'source, 'render>>,
}

#[cfg_attr(test, derive(Debug))]
pub enum State<'source, 'render> {
    /// An entire scope of variables, always a map
    Scope(&'render Value),

    /// A single variable.
    Var(&'source ast::Ident<'source>, ValueCow<'render>),

    /// The current state of a loop iteration
    Loop(LoopState<'source, 'render>),

    /// An expression that we are building
    Expr(ValueCow<'render>),
}

impl<'source, 'render> Stack<'source, 'render> {
    pub fn new(source: &'source str, globals: &'render Value) -> Self {
        Self {
            source,
            stack: vec![State::Scope(globals)],
        }
    }

    /// Resolves a path to a variable on the stack.
    pub fn resolve_path(&self, path: &[ast::Ident<'source>]) -> Result<ValueCow<'render>> {
        'outer: for scope in self.stack.iter().rev() {
            match scope {
                State::Scope(scope) => {
                    let mut v: &Value = scope;
                    for (i, idx) in path.iter().enumerate() {
                        v = match index(self.source(), v, idx) {
                            Ok(d) => d,
                            Err(err) => {
                                // If it is the first segment of the path then
                                // we can try the next state.
                                if i == 0 {
                                    continue 'outer;
                                }
                                return Err(err);
                            }
                        };
                    }

                    return Ok(ValueCow::Borrowed(v));
                }

                State::Var(name, value) if path[0].raw == name.raw => {
                    let mut v: &Value = &value;
                    for p in &path[1..] {
                        v = index(self.source(), v, p)?;
                    }
                    return Ok(ValueCow::Owned(v.clone()));
                }

                State::Loop(loop_state) => {
                    if let Some(value) = loop_state.resolve_path(self.source(), path)? {
                        return Ok(value);
                    }
                }
                _ => {}
            }
        }
        Err(Error::new(
            "not found in this scope",
            self.source(),
            path[0].span,
        ))
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

    pub fn pop_expr(&mut self) -> ValueCow<'render> {
        match self.stack.pop().unwrap() {
            State::Expr(value) => value,
            _ => panic!("expected expression"),
        }
    }

    pub fn source(&self) -> &'source str {
        self.source
    }
}
