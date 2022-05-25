mod iter;
mod stack;

use std::fmt::Write;

use crate::instr::{Instr, Template};
use crate::render::iter::{LoopState, ValueCow};
use crate::render::stack::{Stack, State};
use crate::span::Span;
use crate::{Engine, Error, Result, Value};

/// A renderer that can render a compiled template as a string.
pub struct Renderer<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
}

impl<'engine, 'source> Renderer<'engine, 'source> {
    pub fn new(engine: &'engine Engine<'engine>, template: &'source Template<'source>) -> Self {
        Self { engine, template }
    }

    pub fn render(&self, globals: Value) -> Result<String> {
        let mut buf = String::with_capacity(self.source().len());

        let mut pc = 0;
        let mut stack = Stack::new(self.template.source, &globals);

        while let Some(instr) = self.template.instrs.get(pc) {
            match instr {
                Instr::EmitRaw(raw) => {
                    buf.push_str(raw);
                }

                Instr::StartLoop(vars, span) => {
                    let iterable = stack.pop_expr();
                    stack.push(State::Loop(LoopState::new(
                        self.source(),
                        vars,
                        iterable,
                        *span,
                    )?));
                }

                Instr::Iterate(j) => {
                    let result = stack.last_loop_state_mut().iterate();
                    if result.is_none() {
                        stack.pop_loop_state();
                        pc = *j;
                        continue;
                    }
                }

                Instr::Jump(j) => {
                    pc = *j;
                    continue;
                }

                Instr::JumpIfFalse(j, span) => {
                    let value = stack.pop_expr();
                    let b = match &*value {
                        Value::Bool(cond) => *cond,
                        value => {
                            return Err(Error::new(
                                format!(
                                    "expected bool, but expression evaluated to {}",
                                    value.human()
                                ),
                                self.source(),
                                *span,
                            ));
                        }
                    };
                    if !b {
                        pc = *j;
                        continue;
                    }
                }

                Instr::Push(path) => {
                    let value = stack.resolve_path(path)?;
                    stack.push(State::Expr(value));
                }

                Instr::PopEmit(span) => {
                    let value = stack.pop_expr();
                    self.render_value(&mut buf, &value, *span)?;
                }

                Instr::Call(name) => {
                    let func = self.engine.filters.get(name.raw).ok_or_else(|| {
                        Error::new("unknown filter function", self.source(), name.span)
                    })?;
                    let value = stack.last_expr_mut();
                    match value {
                        ValueCow::Borrowed(b) => {
                            let mut o = b.clone();
                            (func)(&mut o);
                            *value = ValueCow::Owned(o);
                        }
                        ValueCow::Owned(ref mut o) => (func)(o),
                    }
                }
            }
            pc += 1;
        }

        assert!(pc == self.template.instrs.len());

        Ok(buf)
    }

    fn render_value(&self, buf: &mut String, value: &Value, span: Span) -> Result<()> {
        match value {
            Value::None => {}
            Value::Bool(b) => write!(buf, "{}", b).unwrap(),
            Value::Integer(n) => write!(buf, "{}", n).unwrap(),
            Value::Float(n) => write!(buf, "{}", n).unwrap(),
            Value::String(s) => write!(buf, "{}", s).unwrap(),
            val => {
                return Err(Error::new(
                    format!(
                        "expected renderable value, but expression evaluated to {}",
                        val.human()
                    ),
                    self.source(),
                    span,
                ));
            }
        }
        Ok(())
    }

    fn source(&self) -> &'source str {
        self.template.source
    }
}

impl Value {
    fn human(&self) -> &'static str {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
    }
}
