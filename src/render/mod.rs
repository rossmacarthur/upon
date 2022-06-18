mod fmt;
mod iter;
mod stack;
mod value;

use std::fmt::Write;
use std::io;

use crate::render::fmt::{Formatter, Writer};
use crate::render::iter::LoopState;
use crate::render::stack::{Stack, State};
use crate::types::program::{Instr, Template};
use crate::{Engine, Error, Result, Value};

pub fn template<'engine, 'source>(
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
    globals: Value,
) -> Result<String> {
    Renderer::new(engine, template).render(globals)
}

pub fn template_to<'engine, 'source, W>(
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
    writer: W,
    globals: Value,
) -> Result<()>
where
    W: io::Write,
{
    Renderer::new(engine, template).render_to(writer, globals)
}

/// A renderer that interprets a compiled [`Template`].
#[cfg_attr(test, derive(Debug))]
struct Renderer<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
}

impl<'engine, 'source> Renderer<'engine, 'source> {
    pub fn new(engine: &'engine Engine<'engine>, template: &'source Template<'source>) -> Self {
        Self { engine, template }
    }

    pub fn render(&self, globals: Value) -> Result<String> {
        let mut s = String::with_capacity(self.source().len());
        let mut f = Formatter::with_string(&mut s);
        self.render_impl(&mut f, globals)?;
        Ok(s)
    }

    pub fn render_to<W>(&self, writer: W, globals: Value) -> Result<()>
    where
        W: io::Write,
    {
        let mut w = Writer::new(writer);
        let mut f = Formatter::with_writer(&mut w);
        self.render_impl(&mut f, globals)
            .map_err(|err| w.take_err().map(Error::from).unwrap_or(err))
    }

    fn render_impl(&self, f: &mut Formatter<'_>, globals: Value) -> Result<()> {
        let mut pc = 0;
        let mut stack = Stack::new(self.template.source, &globals);

        while let Some(instr) = self.template.instrs.get(pc) {
            match instr {
                Instr::EmitRaw(raw) => {
                    f.write_str(raw)?;
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

                Instr::JumpIfTrue(j, span) => {
                    if stack.pop_expr().as_bool(self.source(), *span)? {
                        pc = *j;
                        continue;
                    }
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
                    default_formatter(f, &value)
                        .map_err(|err| err.with_span(self.source(), *span))?;
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

        Ok(())
    }

    fn source(&self) -> &'source str {
        self.template.source
    }
}

#[inline]
fn default_formatter(f: &mut Formatter<'_>, value: &Value) -> Result<()> {
    match value {
        Value::None => {}
        Value::Bool(b) => write!(f, "{}", b)?,
        Value::Integer(n) => write!(f, "{}", n)?,
        Value::Float(n) => write!(f, "{}", n)?,
        Value::String(s) => write!(f, "{}", s)?,
        value => {
            Err(format!(
                "expected renderable value, but expression evaluated to {}",
                value.human()
            ))?;
        }
    }
    Ok(())
}
