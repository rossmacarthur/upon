use crate::render::iter::LoopState;
use crate::render::value::{index, ValueCow};
use crate::types::ast;
use crate::{Error, Result, Value};

pub struct Stack<'source, 'render> {
    source: &'source str,
    stack: Vec<State<'source, 'render>>,
}

pub enum State<'source, 'render> {
    /// An entire scope of variables, always a map
    Scope(&'render Value),

    /// An expression that we are building
    Expr(ValueCow<'render>),

    /// The current state of a loop iteration
    Loop(LoopState<'source, 'render>),
}

impl<'source, 'render> Stack<'source, 'render> {
    pub fn new(source: &'source str, globals: &'render Value) -> Self {
        Self {
            source,
            stack: vec![State::Scope(globals)],
        }
    }

    /// Resolves a path to a variable on the stack.
    pub fn resolve_path(&self, path: &ast::Path<'source>) -> Result<ValueCow<'render>> {
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

    pub fn last_expr_mut(&mut self) -> &mut ValueCow<'render> {
        match self.stack.last_mut().unwrap() {
            State::Expr(value) => value,
            _ => panic!("expected expression"),
        }
    }

    pub fn last_loop_state_mut(&mut self) -> &mut LoopState<'source, 'render> {
        match self.stack.last_mut().unwrap() {
            State::Loop(loop_state) => loop_state,
            _ => panic!("expected loop state"),
        }
    }

    pub fn pop_expr(&mut self) -> ValueCow<'render> {
        match self.stack.pop().unwrap() {
            State::Expr(value) => value,
            _ => panic!("expected expression"),
        }
    }

    pub fn pop_loop_state(&mut self) -> LoopState<'source, 'render> {
        match self.stack.pop().unwrap() {
            State::Loop(state) => state,
            _ => panic!("expected loop state"),
        }
    }

    fn source(&self) -> &'source str {
        self.source
    }
}
