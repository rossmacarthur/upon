//! AST for a template.

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Template<'t> {
    pub source: &'t str,
    pub scope: Scope<'t>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<'t> {
    pub stmts: Vec<Stmt<'t>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'t> {
    Raw(&'t str),
    InlineExpr(InlineExpr<'t>),
    IfElse(IfElse<'t>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineExpr<'t> {
    pub expr: Expr<'t>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElse<'t> {
    pub cond: Expr<'t>,
    pub then_branch: Scope<'t>,
    pub else_branch: Option<Scope<'t>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'t> {
    Var(Var<'t>),
    Call(Call<'t>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var<'t> {
    pub path: Vec<Ident<'t>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'t> {
    pub name: Ident<'t>,
    pub receiver: Box<Expr<'t>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'t> {
    pub ident: &'t str,
    pub span: Span,
}

impl Scope<'_> {
    pub const fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}

impl Expr<'_> {
    pub const fn span(&self) -> Span {
        match self {
            Expr::Var(var) => var.span,
            Expr::Call(call) => call.span,
        }
    }
}
