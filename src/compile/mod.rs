//! Compile the template into a program that can be executed by the renderer.
//!
//! This process has three stages:
//! - The lexer chunks the template source into tokens.
//! - The parser constructs an AST from the token stream.
//! - The compiler takes the AST and constructs the program.

mod lex;
mod parse;

use crate::types::ast;
use crate::types::program::{Instr, Template, FIXME};
use crate::{Engine, Result};

/// Compile a template into a program.
pub fn template<'engine, 'source>(
    engine: &'engine Engine<'engine>,
    source: &'source str,
) -> Result<Template<'source>> {
    let ast = parse::Parser::new(engine, source).parse_template()?;
    Ok(Compiler::new().compile_template(ast))
}

/// A compiler that constructs a program from an AST.
struct Compiler<'source> {
    instrs: Vec<Instr<'source>>,
}

impl<'source> Compiler<'source> {
    fn new() -> Self {
        Self { instrs: Vec::new() }
    }

    fn compile_template(mut self, template: ast::Template<'source>) -> Template<'source> {
        let ast::Template { source, scope } = template;
        self.compile_scope(scope);
        Template {
            source,
            instrs: self.instrs,
        }
    }

    fn compile_scope(&mut self, scope: ast::Scope<'source>) {
        for stmt in scope.stmts {
            self.compile_stmt(stmt);
        }
    }

    fn compile_stmt(&mut self, stmt: ast::Stmt<'source>) {
        match stmt {
            ast::Stmt::Raw(raw) => {
                self.push(Instr::EmitRaw(raw));
            }

            ast::Stmt::InlineExpr(ast::InlineExpr { expr, .. }) => {
                let span = expr.span();
                self.compile_expr(expr);
                self.push(Instr::PopEmit(span));
            }

            ast::Stmt::IfElse(ast::IfElse {
                not,
                cond,
                then_branch,
                else_branch,
            }) => {
                let span = cond.span();
                self.compile_expr(cond);

                // then branch
                let instr = if not {
                    Instr::JumpIfTrue(FIXME, span)
                } else {
                    Instr::JumpIfFalse(FIXME, span)
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
                self.push(Instr::StartLoop(vars, span));
                let j = self.push(Instr::Iterate(FIXME));
                self.compile_scope(body);
                self.push(Instr::Jump(j));
                self.update_jump(j);
            }
        }
    }

    fn compile_expr(&mut self, expr: ast::Expr<'source>) {
        match expr {
            ast::Expr::Var(ast::Var { path, .. }) => {
                self.push(Instr::Push(path));
            }

            ast::Expr::Call(ast::Call { name, receiver, .. }) => {
                self.compile_expr(*receiver);
                self.push(Instr::Call(name));
            }
        }
    }

    fn update_jump(&mut self, i: usize) {
        let n = self.instrs.len();
        let j = match &mut self.instrs[i] {
            Instr::Jump(j)
            | Instr::JumpIfTrue(j, _)
            | Instr::JumpIfFalse(j, _)
            | Instr::Iterate(j) => j,
            _ => panic!("not a jump instr"),
        };
        *j = n;
    }

    fn push(&mut self, instr: Instr<'source>) -> usize {
        let i = self.instrs.len();
        self.instrs.push(instr);
        i
    }
}
