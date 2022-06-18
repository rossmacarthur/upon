mod fmt;
mod iter;
mod stack;
mod value;

use std::fmt::Write;
use std::io;

pub use crate::render::fmt::Formatter;

use crate::render::fmt::Writer;
use crate::render::iter::LoopState;
use crate::render::stack::{Stack, State};
use crate::types::program::{Instr, Template};
use crate::{Engine, EngineFn, Error, Result, Value};

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
                    // Emit the value using the default formatter.
                    (self.engine.default_formatter)(f, &value)
                        .map_err(|err| err.with_span(self.source(), *span))?;
                }

                Instr::PopEmitWith(name, span) => {
                    match self.engine.functions.get(name.raw) {
                        // The referenced function is a filter, so we apply
                        // it and then emit the value using the default
                        // formatter.
                        Some(EngineFn::Filter(filter)) => {
                            let mut value = stack.pop_expr();
                            value.apply(&**filter);
                            (self.engine.default_formatter)(f, &value)
                                .map_err(|err| err.with_span(self.source(), *span))?;
                        }
                        // The referenced function is a formatter so we simply
                        // emit the value with it.
                        Some(EngineFn::Formatter(formatter)) => {
                            let value = stack.pop_expr();
                            formatter(f, &value)
                                .map_err(|err| err.with_span(self.source(), *span))?;
                        }
                        // No filter or formatter exists.
                        None => {
                            return Err(Error::new(
                                "unknown filter or formatter",
                                self.source(),
                                name.span,
                            ));
                        }
                    }
                }

                Instr::Call(name) => match self.engine.functions.get(name.raw) {
                    // The referenced function is a filter, so we apply it.
                    Some(EngineFn::Filter(filter)) => {
                        stack.last_expr_mut().apply(&**filter);
                    }
                    // The referenced function is a formatter which is not valid
                    // in the middle of an expression.
                    Some(EngineFn::Formatter(_)) => {
                        return Err(Error::new(
                            "expected filter, found formatter",
                            self.source(),
                            name.span,
                        ));
                    }
                    // No filter or formatter exists.
                    None => {
                        return Err(Error::new("unknown filter", self.source(), name.span));
                    }
                },
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

/// The default value formatter.
///
/// Values are formatted as follows:
/// - [`Value::None`]: empty string
/// - [`Value::Bool`]: `true` or `false`
/// - [`Value::Integer`]: the integer formatted using [`Display`][std::fmt::Display]
/// - [`Value::Float`]: the float formatted using [`Display`][std::fmt::Display]
/// - [`Value::String`]: the string, unescaped
///
/// This is public so that it can be called as part of custom formatters.
#[inline]
pub fn format(f: &mut Formatter<'_>, value: &Value) -> Result<()> {
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
