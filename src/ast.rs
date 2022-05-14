//! AST for a template.

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Template<'source> {
    pub source: &'source str,
    pub scope: Scope<'source>,
}

#[derive(Debug, Clone)]
pub struct Scope<'source> {
    pub stmts: Vec<Stmt<'source>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'source> {
    Raw(&'source str),
    InlineExpr(InlineExpr<'source>),
    IfElse(IfElse<'source>),
    ForLoop(ForLoop<'source>),
}

#[derive(Debug, Clone)]
pub struct InlineExpr<'source> {
    pub expr: Expr<'source>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfElse<'source> {
    pub cond: Expr<'source>,
    pub then_branch: Scope<'source>,
    pub else_branch: Option<Scope<'source>>,
}

#[derive(Debug, Clone)]
pub struct ForLoop<'source> {
    pub vars: LoopVars<'source>,
    pub iterable: Expr<'source>,
    pub body: Scope<'source>,
}

#[derive(Debug, Clone)]
pub enum LoopVars<'source> {
    Item(Ident<'source>),
    KeyValue(KeyValue<'source>),
}

#[derive(Debug, Clone)]
pub struct KeyValue<'source> {
    pub key: Ident<'source>,
    pub value: Ident<'source>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr<'source> {
    Var(Var<'source>),
    Call(Call<'source>),
}

#[derive(Debug, Clone)]
pub struct Var<'source> {
    pub path: Vec<Ident<'source>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Call<'source> {
    pub name: Ident<'source>,
    pub receiver: Box<Expr<'source>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Ident<'source> {
    pub raw: &'source str,
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
