use std::fmt::Write;

use crate::fmt::Formatter;
use crate::render::iter::LoopState;
use crate::render::stack::{Stack, State};
use crate::render::RendererInner;
use crate::types::ast;
use crate::types::program::{Instr, Template};
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{EngineBoxCallable, Error, Result, Value};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct RendererImpl<'render, 'stack> {
    pub(crate) inner: RendererInner<'render>,
    pub(crate) stack: Stack<'stack>,
}

#[cfg(feature = "functions")]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct FunctionState<'stack, 'args>
where
    'stack: 'args,
{
    pub source: &'stack str,
    pub fname: &'args str,
    pub args: &'args mut [(ValueCow<'stack>, Span)],
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
                    let name = Some(template_name.as_str());
                    templates.push((template, name, 0, false));
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
                    let name = Some(template_name.as_str());
                    templates.push((template, name, 0, true));
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
        // The expressions that we are building
        let mut exprs: Vec<(ValueCow<'stack>, Span)> = Vec::new();

        while let Some(instr) = t.instrs.get(*pc) {
            match instr {
                Instr::Jump(j) => {
                    *pc = *j;
                    continue;
                }

                Instr::JumpIfTrue(j) => {
                    if exprs.pop().unwrap().0.as_bool() {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::JumpIfFalse(j) => {
                    if !exprs.pop().unwrap().0.as_bool() {
                        *pc = *j;
                        continue;
                    }
                }

                Instr::Emit => {
                    let (value, span) = exprs.pop().unwrap();
                    (self.inner.engine.default_formatter)(f, &value)
                        .map_err(|err| Error::format(err, &t.source, span))?;
                }

                Instr::EmitRaw(span) => {
                    let raw = &t.source[*span];
                    // We don't need to enrich this error because it can only
                    // fail because of an IO error.
                    f.write_str(raw)?;
                }

                Instr::EmitWith(name, _arity, _span) => {
                    let fname = &t.source[name.span];
                    match self.inner.engine.callables.get(fname) {
                        // The referenced function is a formatter so we simply
                        // emit the value with it.
                        Some(EngineBoxCallable::Formatter(formatter)) => {
                            let (value, _) = exprs.pop().unwrap();
                            formatter(f, &value)
                                .map_err(|err| Error::format(err, &t.source, name.span))?;
                        }
                        // The referenced function is a function, so we apply
                        // it and then emit the value using the default
                        // formatter.
                        #[cfg(feature = "functions")]
                        Some(EngineBoxCallable::Function(function)) => {
                            let at = exprs.len() - _arity;
                            let args = &mut exprs[at..];
                            let result = function(FunctionState {
                                source: &t.source,
                                fname,
                                args,
                            })
                            .map_err(|err| err.enrich(&t.source, name.span))?;
                            exprs.truncate(at);
                            (self.inner.engine.default_formatter)(f, &result)
                                .map_err(|err| Error::format(err, &t.source, *_span))?;
                        }
                        // No formatter or function exists.
                        None => {
                            return Err(Error::render(
                                "unknown formatter or function",
                                &t.source,
                                name.span,
                            ));
                        }
                    }
                }

                Instr::LoopStart(vars, span) => {
                    let (iterable, _) = exprs.pop().unwrap();
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
                    let (value, _) = exprs.pop().unwrap();
                    self.stack.push(State::Var(name, value))
                }

                Instr::WithEnd => {
                    self.stack.pop_var();
                }

                Instr::Include(template_name) => {
                    *pc += 1;
                    debug_assert!(exprs.is_empty());
                    return Ok(RenderState::Include { template_name });
                }

                Instr::IncludeWith(template_name) => {
                    *pc += 1;
                    let (globals, _) = exprs.pop().unwrap();
                    debug_assert!(exprs.is_empty());
                    return Ok(RenderState::IncludeWith {
                        template_name,
                        globals,
                    });
                }

                Instr::ExprStartVar(var) => {
                    let value = self.stack.lookup_var(&t.source, var)?;
                    exprs.push((value, var.span()));
                }

                Instr::ExprStartLiteral(literal) => {
                    let value = ValueCow::Borrowed(&literal.value);
                    exprs.push((value, literal.span));
                }

                Instr::ExprStartList(span) => {
                    let value = ValueCow::Owned(crate::Value::new_list());
                    exprs.push((value, *span));
                }

                Instr::ExprStartMap(span) => {
                    let value = ValueCow::Owned(crate::Value::new_map());
                    exprs.push((value, *span));
                }

                Instr::ExprListPush => {
                    let (item, _) = exprs.pop().unwrap();
                    match exprs.last_mut().unwrap() {
                        (ValueCow::Owned(Value::List(l)), _) => {
                            l.push(item.to_owned());
                        }
                        _ => panic!("expected owned list"),
                    }
                }

                Instr::ExprMapInsert(key) => {
                    let key = key.value.clone();
                    let (value, _) = exprs.pop().unwrap();
                    match exprs.last_mut().unwrap() {
                        (ValueCow::Owned(Value::Map(m)), _) => {
                            m.insert(key, value.to_owned());
                        }
                        _ => panic!("expected owned map"),
                    }
                }

                Instr::Apply(name, _arity, _span) => {
                    let fname = &t.source[name.span];
                    match self.inner.engine.callables.get(fname) {
                        // The referenced function is a formatter which is not valid
                        // in the middle of an expression.
                        Some(EngineBoxCallable::Formatter(_)) => {
                            return Err(Error::render(
                                "expected function, found formatter",
                                &t.source,
                                name.span,
                            ));
                        }
                        // The referenced function is a function, so we apply it.
                        #[cfg(feature = "functions")]
                        Some(EngineBoxCallable::Function(function)) => {
                            let at = exprs.len() - _arity;
                            let args = &mut exprs[at..];
                            let result = function(FunctionState {
                                source: &t.source,
                                fname,
                                args,
                            })
                            .map_err(|e| e.enrich(&t.source, *_span))?;
                            exprs.truncate(at);
                            exprs.push((ValueCow::Owned(result), *_span));
                        }
                        // No formatter or function exists.
                        None => {
                            return Err(Error::render("unknown function", &t.source, name.span));
                        }
                    }
                }
            }
            *pc += 1;
        }

        assert!(*pc == t.instrs.len());
        debug_assert!(exprs.is_empty());
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
