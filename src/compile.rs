use crate::ast;
use crate::instr::{Instr, Template, FIXME};

/// A compiler that constructs a program from the AST.
pub struct Compiler<'source> {
    instrs: Vec<Instr<'source>>,
}

impl<'source> Compiler<'source> {
    pub fn new() -> Self {
        Self { instrs: Vec::new() }
    }

    pub fn compile_template(mut self, template: ast::Template<'source>) -> Template<'source> {
        let ast::Template { source, scope } = template;
        self.compile_scope(scope);
        Template {
            source,
            instrs: self.instrs,
            spans: Vec::new(),
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
                cond,
                then_branch,
                else_branch,
            }) => {
                let span = cond.span();
                self.compile_expr(cond);

                // then branch
                let j = self.push(Instr::JumpIfFalse(FIXME, span));
                self.compile_scope(then_branch);

                if let Some(else_branch) = else_branch {
                    // else branch
                    let j2 = self.push(Instr::Jump(FIXME));
                    self.update_jump(j);
                    self.compile_scope(else_branch);
                    self.update_jump(j2)
                } else {
                    self.update_jump(j);
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
            Instr::Jump(j) | Instr::JumpIfFalse(j, _) | Instr::Iterate(j) => j,
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
