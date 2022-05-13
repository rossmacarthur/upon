//! AST for a template.

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Template<'t> {
    pub source: &'t str,
    pub scope: Scope<'t>,
}

#[derive(Debug, Clone)]
pub struct Scope<'t> {
    pub stmts: Vec<Stmt<'t>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'t> {
    Raw(&'t str),
    InlineExpr(InlineExpr<'t>),
    IfElse(IfElse<'t>),
    ForLoop(ForLoop<'t>),
}

#[derive(Debug, Clone)]
pub struct InlineExpr<'t> {
    pub expr: Expr<'t>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfElse<'t> {
    pub cond: Expr<'t>,
    pub then_branch: Scope<'t>,
    pub else_branch: Option<Scope<'t>>,
}

#[derive(Debug, Clone)]
pub struct ForLoop<'t> {
    pub vars: LoopVars<'t>,
    pub iterable: Expr<'t>,
    pub body: Scope<'t>,
}

#[derive(Debug, Clone)]
pub enum LoopVars<'t> {
    Item(Ident<'t>),
    KeyValue(KeyValue<'t>),
}

#[derive(Debug, Clone)]
pub struct KeyValue<'t> {
    pub key: Ident<'t>,
    pub value: Ident<'t>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr<'t> {
    Var(Var<'t>),
    Call(Call<'t>),
}

#[derive(Debug, Clone)]
pub struct Var<'t> {
    pub path: Vec<Ident<'t>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Call<'t> {
    pub name: Ident<'t>,
    pub receiver: Box<Expr<'t>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Ident<'t> {
    pub value: &'t str,
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
            Self::Var(var) => var.span,
            Self::Call(call) => call.span,
        }
    }
}
