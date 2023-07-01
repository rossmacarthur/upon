//! Compile the template into a program that can be executed by the renderer.
//!
//! This process has three stages:
//! - The lexer chunks the template source into tokens.
//! - The parser constructs an AST from the token stream.
//! - The compiler takes the AST and constructs the program.

mod lex;
mod parse;
mod search;

use std::borrow::Cow;

pub use crate::compile::search::Searcher;

use crate::types::ast;
use crate::types::program::{Instr, Template, FIXME};
use crate::types::span::Span;
use crate::{Engine, Result};

/// Compile a template into a program.
pub fn template<'engine, 'source>(
    engine: &'engine Engine<'engine>,
    source: Cow<'source, str>,
) -> Result<Template<'source>> {
    let ast = parse::Parser::new(engine, &source).parse_template()?;
    Ok(Compiler::new().compile_template(source, ast))
}

/// A compiler that constructs a program from an AST.
#[cfg_attr(internal_debug, derive(Debug))]
struct Compiler {
    instrs: Vec<Instr>,
}

impl Compiler {
    fn new() -> Self {
        Self { instrs: Vec::new() }
    }

    fn compile_template(mut self, source: Cow<'_, str>, template: ast::Template) -> Template<'_> {
        let ast::Template { scope } = template;
        self.compile_scope(scope);
        Template {
            source,
            instrs: self.instrs,
        }
    }

    fn compile_scope(&mut self, scope: ast::Scope) {
        for stmt in scope.stmts {
            self.compile_stmt(stmt);
        }
    }

    fn compile_stmt(&mut self, stmt: ast::Stmt) {
        match stmt {
            ast::Stmt::Raw(raw) => {
                self.push(Instr::EmitRaw(raw));
            }

            ast::Stmt::InlineExpr(ast::InlineExpr { expr, .. }) => {
                let span = expr.span();
                self.compile_expr(expr);
                self.pop_emit_expr(span);
            }

            ast::Stmt::Include(ast::Include { name, globals }) => match globals {
                Some(globals) => {
                    self.compile_expr(globals);
                    self.push(Instr::IncludeWith(name));
                }
                None => {
                    self.push(Instr::Include(name));
                }
            },

            ast::Stmt::IfElse(ast::IfElse {
                not,
                cond,
                then_branch,
                else_branch,
            }) => {
                self.compile_expr(cond);

                // then branch
                let instr = if not {
                    Instr::JumpIfTrue(FIXME)
                } else {
                    Instr::JumpIfFalse(FIXME)
                };
                let j = self.push(instr);
                self.compile_scope(then_branch);

                match else_branch {
                    Some(else_branch) => {
                        // else branch
                        let j2 = self.push(Instr::Jump(FIXME));
                        self.update_jump(j);
                        self.compile_scope(else_branch);
                        self.update_jump(j2)
                    }
                    None => {
                        self.update_jump(j);
                    }
                }
            }

            ast::Stmt::ForLoop(ast::ForLoop {
                vars,
                iterable,
                body,
            }) => {
                let span = iterable.span();
                self.compile_expr(iterable);
                self.push(Instr::LoopStart(vars, span));
                let j = self.push(Instr::LoopNext(FIXME));
                self.compile_scope(body);
                self.push(Instr::Jump(j));
                self.update_jump(j);
            }

            ast::Stmt::With(ast::With { expr, name, body }) => {
                self.compile_expr(expr);
                self.push(Instr::WithStart(name));
                self.compile_scope(body);
                self.push(Instr::WithEnd);
            }
        }
    }

    fn compile_expr(&mut self, expr: ast::Expr) {
        match expr {
            ast::Expr::Base(base_expr) => {
                self.compile_base_expr(base_expr);
            }

            ast::Expr::Call(ast::Call {
                name,
                args,
                receiver,
                span,
            }) => {
                self.compile_expr(*receiver);
                self.push(Instr::Apply(name, span, args));
            }
        }
    }

    fn compile_base_expr(&mut self, base_expr: ast::BaseExpr) {
        match base_expr {
            ast::BaseExpr::Var(var) => {
                self.push(Instr::ExprStart(var));
            }
            ast::BaseExpr::Literal(ast::Literal { value, .. }) => {
                self.push(Instr::ExprStartLit(value));
            }
        }
    }

    fn pop_emit_expr(&mut self, span: Span) {
        let emit = match self.instrs.last() {
            Some(Instr::Apply(_, _, None)) => {
                let instr = self.instrs.pop().unwrap();
                match instr {
                    Instr::Apply(ident, _, _) => Instr::EmitWith(ident, span),
                    _ => unreachable!(),
                }
            }
            _ => Instr::Emit(span),
        };
        self.push(emit);
    }

    fn update_jump(&mut self, i: usize) {
        let n = self.instrs.len();
        let j = match &mut self.instrs[i] {
            Instr::Jump(j) | Instr::JumpIfTrue(j) | Instr::JumpIfFalse(j) | Instr::LoopNext(j) => j,
            _ => panic!("not a jump instr"),
        };
        *j = n;
    }

    fn push(&mut self, instr: Instr) -> usize {
        let i = self.instrs.len();
        self.instrs.push(instr);
        i
    }
}
