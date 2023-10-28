use std::fmt::Write;

use crate::fmt::Formatter;
use crate::render::iter::LoopState;
use crate::render::stack::{Stack, State};
use crate::render::RendererInner;
use crate::types::ast;
use crate::types::program::{Instr, Template};
use crate::value::ValueCow;
use crate::{EngineBoxFn, Error, Result};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct RendererImpl<'render, 'stack> {
    pub(crate) inner: RendererInner<'render>,
    pub(crate) stack: Stack<'stack>,
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
enum RenderState<'render, 'stack> {
    Done,
    Include {
        template_name: &'render ast::String,
    },
    IncludeWith {
        template_name: &'render ast::String,
        globals: ValueCow<'stack>,
    },
}

impl<'render, 'stack> RendererImpl<'render, 'stack>
where
    'render: 'stack,
{
    pub(crate) fn render(mut self, f: &mut Formatter<'_>) -> Result<()> {
        let mut templates = vec![(self.inner.template, self.inner.template_name, 0, false)];

        let max_include_depth = self
            .inner
            .max_include_depth
            .unwrap_or(self.inner.engine.max_include_depth);

        while let Some((t, tname, pc, has_scope)) = templates.last_mut() {
            let state = self.render_one(f, t, pc).map_err(|e| match tname {
                Some(s) => e.with_template_name(s.to_owned()),
                None => e,
            })?;
            match state {
                RenderState::Done => {
                    if *has_scope {
                        self.stack.pop_scope();
                        self.stack.pop_boundary();
                    }
                    templates.pop();
                }
                RenderState::Include { template_name } => {
                    let template =
                        self.get_template(&t.source, template_name)
                            .map_err(|e| match tname {
                                Some(s) => e.with_template_name(s.to_owned()),
                                None => e,
                            })?;
                    templates.push((template, Some(template_name.as_str()), 0, false));
                }
                RenderState::IncludeWith {
                    template_name,
                    globals,
                } => {
                    let template =
                        self.get_template(&t.source, template_name)
                            .map_err(|e| match tname {
                                Some(s) => e.with_template_name(s.to_owned()),
                                None => e,
                            })?;
                    self.stack.push(State::Boundary);
                    self.stack.push(State::Scope(globals));
                    templates.push((template, Some(template_name.as_str()), 0, true));
                }
            }
            if templates.len() > max_include_depth {
                return Err(Error::max_include_depth(max_include_depth));
            }
        }

        Ok(())
    }

    fn render_one(
        &mut self,
        f: &mut Formatter<'_>,
        t: &'render Template<'render>,
        pc: &mut usize,
    ) -> Result<RenderState<'render, 'stack>> {
        // An expression that we are building
        let mut expr: Option<ValueCow<'stack>> = None;

        while let Some(instr) = t.instrs.get(*pc) {
            match instr {
                Instr::Jump(j) => {
                    *pc = *j;
                    continue;
                }

                Instr::JumpIfTrue(j) => {
                    if expr.take().unwrap().as_bool() {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::JumpIfFalse(j) => {
                    if !expr.take().unwrap().as_bool() {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::Emit(span) => {
                    let value = expr.take().unwrap();
                    (self.inner.engine.default_formatter)(f, &value)
                        .map_err(|err| Error::format(err, &t.source, *span))?;
                }

                Instr::EmitRaw(span) => {
                    let raw = &t.source[*span];
                    // We don't need to enrich this error because it can only
                    // fail because of an IO error.
                    f.write_str(raw)?;
                }

                Instr::EmitWith(name, _span) => {
                    let name_raw = &t.source[name.span];
                    match self.inner.engine.functions.get(name_raw) {
                        // The referenced function is a filter, so we apply
                        // it and then emit the value using the default
                        // formatter.
                        #[cfg(feature = "filters")]
                        Some(EngineBoxFn::Filter(filter)) => {
                            let mut value = expr.take().unwrap();
                            let result = filter(FilterState {
                                stack: &self.stack,
                                source: &t.source,
                                filter: name,
                                value: &mut value,
                                args: &[],
                            })
                            .map_err(|err| err.enrich(&t.source, name.span))?;
                            (self.inner.engine.default_formatter)(f, &result)
                                .map_err(|err| Error::format(err, &t.source, *_span))?;
                        }
                        // The referenced function is a formatter so we simply
                        // emit the value with it.
                        Some(EngineBoxFn::Formatter(formatter)) => {
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
                    self.stack.push(State::Loop(LoopState::new(
                        &t.source, vars, iterable, *span,
                    )?));
                }

                Instr::LoopNext(j) => {
                    if self.stack.last_loop_state_mut().iterate().is_none() {
                        self.stack.pop_loop_state();
                        *pc = *j;
                        continue;
                    }
                }

                Instr::WithStart(name) => {
                    let value = expr.take().unwrap();
                    self.stack.push(State::Var(name, value))
                }

                Instr::WithEnd => {
                    self.stack.pop_var();
                }

                Instr::Include(template_name) => {
                    *pc += 1;
                    return Ok(RenderState::Include { template_name });
                }

                Instr::IncludeWith(template_name) => {
                    *pc += 1;
                    let globals = expr.take().unwrap();
                    return Ok(RenderState::IncludeWith {
                        template_name,
                        globals,
                    });
                }

                Instr::ExprStart(var) => {
                    let value = self.stack.lookup_var(&t.source, var)?;
                    let prev = expr.replace(value);
                    debug_assert!(prev.is_none());
                }

                Instr::ExprStartLit(value) => {
                    let prev = expr.replace(ValueCow::Owned(value.clone()));
                    debug_assert!(prev.is_none());
                }

                Instr::Apply(name, _, _args) => {
                    let name_raw = &t.source[name.span];
                    match self.inner.engine.functions.get(name_raw) {
                        // The referenced function is a filter, so we apply it.
                        #[cfg(feature = "filters")]
                        Some(EngineBoxFn::Filter(filter)) => {
                            let mut value = expr.take().unwrap();
                            let args = _args
                                .as_ref()
                                .map(|args| args.values.as_slice())
                                .unwrap_or(&[]);
                            let result = filter(FilterState {
                                stack: &self.stack,
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
                        Some(EngineBoxFn::Formatter(_)) => {
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

    fn get_template(
        &mut self,
        source: &str,
        name: &ast::String,
    ) -> Result<&'render Template<'render>> {
        if let Some(template_fn) = &mut self.inner.template_fn {
            template_fn(name.as_str())
                .map(|t| &t.template)
                .map_err(|e| Error::render(e, source, name.span))
        } else {
            self.inner
                .engine
                .templates
                .get(name.as_str())
                .ok_or_else(|| Error::render("unknown template", source, name.span))
        }
    }
}
