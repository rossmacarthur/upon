mod fmt;
mod iter;
mod stack;
mod value;

use std::fmt::Write;
use std::io;

pub use crate::render::fmt::Formatter;

use crate::render::fmt::Writer;
use crate::render::iter::LoopState;
pub use crate::render::stack::{Stack, State};
use crate::types::ast;
use crate::types::program::{Instr, Template};
use crate::types::span::{index, Span};
use crate::value::ValueCow;
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

#[cfg(feature = "filters")]
#[cfg_attr(test, derive(Debug))]
pub struct FilterState<'a> {
    pub stack: &'a Stack<'a, 'a>,
    pub source: &'a str,
    pub filter: &'a ast::Ident,
    pub value: &'a mut ValueCow<'a>,
    pub value_span: Span,
    pub args: &'a [ast::BaseExpr],
}

#[cfg_attr(test, derive(Debug))]
enum RenderState<'engine, 'render> {
    Done,
    Include {
        template: &'engine Template<'engine>,
    },
    IncludeWith {
        template: &'engine Template<'engine>,
        globals: ValueCow<'render>,
    },
}

impl<'engine, 'source> Renderer<'engine, 'source> {
    pub fn new(engine: &'engine Engine<'engine>, template: &'source Template<'source>) -> Self {
        Self { engine, template }
    }

    pub fn render(&self, globals: Value) -> Result<String> {
        let mut s = String::with_capacity(self.template.source.len());
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
        let mut stack = Stack::new(ValueCow::Borrowed(&globals));
        let mut templates = vec![(self.template, 0)];

        while let Some((t, pc)) = templates.last_mut() {
            match self.render_one(f, t, pc, &mut stack)? {
                RenderState::Done => {
                    templates.pop();
                }
                RenderState::Include { template } => {
                    templates.push((template, 0));
                }
                RenderState::IncludeWith { template, globals } => {
                    stack.push(State::Boundary);
                    stack.push(State::Scope(globals));
                    templates.push((template, 0));
                }
            }
        }

        Ok(())
    }

    fn render_one<'render>(
        &self,
        f: &mut Formatter<'_>,
        t: &'source Template<'source>,
        pc: &mut usize,
        stack: &mut Stack<'source, 'render>,
    ) -> Result<RenderState<'engine, 'render>> {
        // An expression that we are building
        let mut expr: Option<ValueCow<'render>> = None;

        while let Some(instr) = t.instrs.get(*pc) {
            match instr {
                Instr::Jump(j) => {
                    *pc = *j;
                    continue;
                }

                Instr::JumpIfTrue(j, span) => {
                    if expr.take().unwrap().as_bool(t.source, *span)? {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::JumpIfFalse(j, span) => {
                    if !expr.take().unwrap().as_bool(t.source, *span)? {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::Emit(span) => {
                    let value = expr.take().unwrap();
                    (self.engine.default_formatter)(f, &value)
                        .map_err(|err| err.with_span(t.source, *span))?;
                }

                Instr::EmitRaw(span) => {
                    let raw = unsafe { index(t.source, *span) };
                    f.write_str(raw)?;
                }

                Instr::EmitWith(name, span) => {
                    let name_raw = unsafe { index(t.source, name.span) };
                    match self.engine.functions.get(name_raw) {
                        // The referenced function is a filter, so we apply
                        // it and then emit the value using the default
                        // formatter.
                        #[cfg(feature = "filters")]
                        Some(EngineFn::Filter(filter)) => {
                            let mut value = expr.take().unwrap();
                            let result = filter(FilterState {
                                stack,
                                source: t.source,
                                filter: name,
                                value: &mut value,
                                value_span: *span,
                                args: &[],
                            })?;
                            (self.engine.default_formatter)(f, &result)
                                .map_err(|err| err.with_span(t.source, *span))?;
                        }
                        // The referenced function is a formatter so we simply
                        // emit the value with it.
                        Some(EngineFn::Formatter(formatter)) => {
                            let value = expr.take().unwrap();
                            formatter(f, &value).map_err(|err| err.with_span(t.source, *span))?;
                        }
                        // No filter or formatter exists.
                        None => {
                            return Err(Error::new(
                                "unknown filter or formatter",
                                t.source,
                                name.span,
                            ));
                        }
                    }
                }

                Instr::LoopStart(vars, span) => {
                    let iterable = expr.take().unwrap();
                    stack.push(State::Loop(LoopState::new(
                        t.source, vars, iterable, *span,
                    )?));
                }

                Instr::LoopNext(j) => {
                    if stack.last_loop_state_mut().iterate().is_none() {
                        stack.pop_loop_state();
                        *pc = *j;
                        continue;
                    }
                }

                Instr::WithStart(name) => {
                    let value = expr.take().unwrap();
                    stack.push(State::Var(name, value))
                }

                Instr::WithEnd => {
                    stack.pop_var();
                }

                Instr::Include(name) => {
                    *pc += 1;
                    let template = self.get_template(t.source, name)?;
                    return Ok(RenderState::Include { template });
                }

                Instr::IncludeWith(name) => {
                    *pc += 1;
                    let template = self.get_template(t.source, name)?;
                    let globals = expr.take().unwrap();
                    return Ok(RenderState::IncludeWith { template, globals });
                }

                Instr::ExprStart(path) => {
                    let value = stack.lookup_path(t.source, path)?;
                    let prev = expr.replace(value);
                    debug_assert!(prev.is_none());
                }

                Instr::ExprStartLit(value) => {
                    let prev = expr.replace(ValueCow::Owned(value.clone()));
                    debug_assert!(prev.is_none());
                }

                Instr::Apply(name, span, args) => {
                    let name_raw = unsafe { index(t.source, name.span) };
                    match self.engine.functions.get(name_raw) {
                        // The referenced function is a filter, so we apply it.
                        #[cfg(feature = "filters")]
                        Some(EngineFn::Filter(filter)) => {
                            let mut value = expr.take().unwrap();
                            let args = args
                                .as_ref()
                                .map(|args| args.values.as_slice())
                                .unwrap_or(&[]);
                            let result = filter(FilterState {
                                stack,
                                source: t.source,
                                filter: name,
                                value: &mut value,
                                value_span: *span,
                                args,
                            })?;
                            expr.replace(ValueCow::Owned(result));
                        }
                        // The referenced function is a formatter which is not valid
                        // in the middle of an expression.
                        Some(EngineFn::Formatter(_)) => {
                            return Err(Error::new(
                                "expected filter, found formatter",
                                t.source,
                                name.span,
                            ));
                        }
                        // No filter or formatter exists.
                        None => {
                            return Err(Error::new("unknown filter", t.source, name.span));
                        }
                    }
                }
            }
            *pc += 1;
        }

        assert!(*pc == t.instrs.len());
        Ok(RenderState::Done)
    }

    fn get_template(&self, source: &str, name: &ast::String) -> Result<&'engine Template<'engine>> {
        self.engine
            .templates
            .get(name.name.as_str())
            .ok_or_else(|| Error::new("unknown template", source, name.span))
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
