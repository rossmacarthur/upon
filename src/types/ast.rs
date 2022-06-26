//! AST representing a template.

use crate::types::span::Span;
use crate::Value;

#[cfg_attr(test, derive(Debug))]
pub struct Template<'source> {
    pub source: &'source str,
    pub scope: Scope<'source>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Scope<'source> {
    pub stmts: Vec<Stmt<'source>>,
}

#[cfg_attr(test, derive(Debug))]
pub enum Stmt<'source> {
    Raw(&'source str),
    InlineExpr(InlineExpr<'source>),
    IfElse(IfElse<'source>),
    ForLoop(ForLoop<'source>),
}

#[cfg_attr(test, derive(Debug))]
pub struct InlineExpr<'source> {
    pub expr: Expr<'source>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub struct IfElse<'source> {
    pub not: bool,
    pub cond: Expr<'source>,
    pub then_branch: Scope<'source>,
    pub else_branch: Option<Scope<'source>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct ForLoop<'source> {
    pub vars: LoopVars<'source>,
    pub iterable: Expr<'source>,
    pub body: Scope<'source>,
}

#[cfg_attr(test, derive(Debug))]
pub enum LoopVars<'source> {
    Item(Ident<'source>),
    KeyValue(KeyValue<'source>),
}

#[cfg_attr(test, derive(Debug))]
pub struct KeyValue<'source> {
    pub key: Ident<'source>,
    pub value: Ident<'source>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub enum Expr<'source> {
    Call(Call<'source>),
    Var(Var<'source>),
}

#[cfg_attr(test, derive(Debug))]
pub struct Call<'source> {
    pub name: Ident<'source>,
    pub args: Option<Args<'source>>,
    pub receiver: Box<Expr<'source>>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub struct Args<'source> {
    pub values: Vec<Arg<'source>>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub enum Arg<'source> {
    Var(Var<'source>),
    Literal(Literal),
}

#[cfg_attr(test, derive(Debug))]
pub struct Var<'source> {
    pub path: Vec<Ident<'source>>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub struct Literal {
    pub value: Value,
    pub span: Span,
}

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug))]
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

impl Arg<'_> {
    pub const fn span(&self) -> Span {
        match self {
            Arg::Var(var) => var.span,
            Arg::Literal(lit) => lit.span,
        }
    }
}
