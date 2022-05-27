mod iter;
mod stack;
mod value;

use std::fmt::Write;

use crate::render::iter::LoopState;
use crate::render::stack::{Stack, State};
use crate::types::prog::{Instr, Template};
use crate::types::span::Span;
use crate::{Engine, Error, Result, Value};

pub fn template<'engine, 'source>(
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
    globals: Value,
) -> Result<String> {
    Renderer::new(engine, template).render(globals)
}

/// A renderer that interprets a compiled [`Template`].
struct Renderer<'engine, 'source> {
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
                    if stack.last_loop_state_mut().iterate().is_none() {
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
                    if !stack.pop_expr().as_bool(self.source(), *span)? {
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
                    stack.last_expr_mut().apply(&**func);
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
            value => {
                return Err(Error::new(
                    format!(
                        "expected renderable value, but expression evaluated to {}",
                        value.human()
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
