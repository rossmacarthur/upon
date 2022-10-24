mod iter;
mod stack;
mod value;

use std::fmt::Write;
use std::io;

use crate::fmt::{Formatter, Writer};
use crate::render::iter::LoopState;
pub use crate::render::stack::{Stack, State};
use crate::types::ast;
use crate::types::program::{Instr, Template};
use crate::types::span::index;
use crate::value::ValueCow;
use crate::{Engine, EngineFn, Error, Result, Value};

pub fn template<'a>(
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
    globals: Value,
) -> Result<String> {
    Renderer::new(engine, template).render(globals)
}

pub fn template_to<'a, W>(
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
    writer: W,
    globals: Value,
) -> Result<()>
where
    W: io::Write,
{
    Renderer::new(engine, template).render_to(writer, globals)
}

/// A renderer that interprets a compiled [`Template`].
#[cfg_attr(internal_debug, derive(Debug))]
struct Renderer<'a> {
    engine: &'a Engine<'a>,
    template: &'a Template<'a>,
}

#[cfg(feature = "filters")]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct FilterState<'a> {
    pub stack: &'a Stack<'a>,
    pub source: &'a str,
    pub filter: &'a ast::Ident,
    pub value: &'a mut ValueCow<'a>,
    pub args: &'a [ast::BaseExpr],
}

#[cfg_attr(internal_debug, derive(Debug))]
enum RenderState<'a> {
    Done,
    Include {
        template: &'a Template<'a>,
    },
    IncludeWith {
        template: &'a Template<'a>,
        globals: ValueCow<'a>,
    },
}

impl<'a> Renderer<'a> {
    pub fn new(engine: &'a Engine<'a>, template: &'a Template<'a>) -> Self {
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
        let mut templates = vec![(self.template, 0, false)];

        while let Some((t, pc, has_scope)) = templates.last_mut() {
            match self.render_one(f, t, pc, &mut stack)? {
                RenderState::Done => {
                    if *has_scope {
                        stack.pop_scope();
                        stack.pop_boundary();
                    }
                    templates.pop();
                }
                RenderState::Include { template } => {
                    templates.push((template, 0, false));
                }
                RenderState::IncludeWith { template, globals } => {
                    stack.push(State::Boundary);
                    stack.push(State::Scope(globals));
                    templates.push((template, 0, true));
                }
            }
            if templates.len() > self.engine.max_include_depth {
                return Err(Error::max_include_depth(self.engine.max_include_depth));
            }
        }

        Ok(())
    }

    fn render_one(
        &self,
        f: &mut Formatter<'_>,
        t: &'a Template<'a>,
        pc: &mut usize,
        stack: &mut Stack<'a>,
    ) -> Result<RenderState<'a>> {
        // An expression that we are building
        let mut expr: Option<ValueCow<'a>> = None;

        while let Some(instr) = t.instrs.get(*pc) {
            match instr {
                Instr::Jump(j) => {
                    *pc = *j;
                    continue;
                }

                Instr::JumpIfTrue(j, span) => {
                    if expr.take().unwrap().as_bool(&t.source, *span)? {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::JumpIfFalse(j, span) => {
                    if !expr.take().unwrap().as_bool(&t.source, *span)? {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::Emit(span) => {
                    let value = expr.take().unwrap();
                    (self.engine.default_formatter)(f, &value)
                        .map_err(|err| Error::format(err, &t.source, *span))?;
                }

                Instr::EmitRaw(span) => {
                    let raw = unsafe { index(&t.source, *span) };
                    // We don't need to enrich this error because it can only
                    // fail because of an IO error.
                    f.write_str(raw)?;
                }

                Instr::EmitWith(name, span) => {
                    let name_raw = unsafe { index(&t.source, name.span) };
                    match self.engine.functions.get(name_raw) {
                        // The referenced function is a filter, so we apply
                        // it and then emit the value using the default
                        // formatter.
                        #[cfg(feature = "filters")]
                        Some(EngineFn::Filter(filter)) => {
                            let mut value = expr.take().unwrap();
                            let result = filter(FilterState {
                                stack,
                                source: &t.source,
                                filter: name,
                                value: &mut value,
                                args: &[],
                            })
                            .map_err(|err| err.enrich(&t.source, name.span))?;
                            (self.engine.default_formatter)(f, &result)
                                .map_err(|err| Error::format(err, &t.source, *span))?;
                        }
                        // The referenced function is a formatter so we simply
                        // emit the value with it.
                        Some(EngineFn::Formatter(formatter)) => {
                            let value = expr.take().unwrap();
                            formatter(f, &value)
                                .map_err(|err| Error::format(err, &t.source, name.span))?;
                        }
                        // No filter or formatter exists.
                        None => {
                            return Err(Error::render(
                                "unknown filter or formatter",
                                &t.source,
                                name.span,
                            ));
                        }
                    }
                }

                Instr::LoopStart(vars, span) => {
                    let iterable = expr.take().unwrap();
                    stack.push(State::Loop(LoopState::new(
                        &t.source, vars, iterable, *span,
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
                    let template = self.get_template(&t.source, name)?;
                    return Ok(RenderState::Include { template });
                }

                Instr::IncludeWith(name) => {
                    *pc += 1;
                    let template = self.get_template(&t.source, name)?;
                    let globals = expr.take().unwrap();
                    return Ok(RenderState::IncludeWith { template, globals });
                }

                Instr::ExprStart(path) => {
                    let value = stack.lookup_path(&t.source, path)?;
                    let prev = expr.replace(value);
                    debug_assert!(prev.is_none());
                }

                Instr::ExprStartLit(value) => {
                    let prev = expr.replace(ValueCow::Owned(value.clone()));
                    debug_assert!(prev.is_none());
                }

                Instr::Apply(name, _, args) => {
                    let name_raw = unsafe { index(&t.source, name.span) };
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
                                source: &t.source,
                                filter: name,
                                value: &mut value,
                                args,
                            })
                            .map_err(|e| e.enrich(&t.source, name.span))?;
                            expr.replace(ValueCow::Owned(result));
                        }
                        // The referenced function is a formatter which is not valid
                        // in the middle of an expression.
                        Some(EngineFn::Formatter(_)) => {
                            return Err(Error::render(
                                "expected filter, found formatter",
                                &t.source,
                                name.span,
                            ));
                        }
                        // No filter or formatter exists.
                        None => {
                            return Err(Error::render("unknown filter", &t.source, name.span));
                        }
                    }
                }
            }
            *pc += 1;
        }

        assert!(*pc == t.instrs.len());
        Ok(RenderState::Done)
    }

    fn get_template(&self, source: &str, name: &ast::String) -> Result<&'a Template<'a>> {
        self.engine
            .templates
            .get(name.name.as_str())
            .ok_or_else(|| Error::render("unknown template", source, name.span))
    }
}
