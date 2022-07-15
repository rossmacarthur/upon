//! AST representing a template.

use crate::types::span::Span;

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
    Include(Include<'source>),
    IfElse(IfElse<'source>),
    ForLoop(ForLoop<'source>),
    With(With<'source>),
}

#[cfg_attr(test, derive(Debug))]
pub struct InlineExpr<'source> {
    pub expr: Expr<'source>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub struct Include<'source> {
    pub name: String,
    pub globals: Option<Expr<'source>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct String {
    pub name: std::string::String,
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
pub struct With<'source> {
    pub expr: Expr<'source>,
    pub name: Ident<'source>,
    pub body: Scope<'source>,
}

#[cfg_attr(test, derive(Debug))]
pub enum Expr<'source> {
    Base(BaseExpr<'source>),
    Call(Call<'source>),
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
    pub values: Vec<BaseExpr<'source>>,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub enum BaseExpr<'source> {
    Var(Var<'source>),
    Literal(Literal),
}

#[cfg_attr(test, derive(Debug))]
pub struct Var<'source> {
    pub path: Vec<Ident<'source>>,
    pub span: Span,
}

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug))]
pub struct Ident<'source> {
    pub raw: &'source str,
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
pub struct Literal {
    pub value: crate::Value,
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
            Self::Base(base) => base.span(),
            Self::Call(call) => call.span,
        }
    }
}

impl BaseExpr<'_> {
    pub const fn span(&self) -> Span {
        match self {
            BaseExpr::Var(var) => var.span,
            BaseExpr::Literal(lit) => lit.span,
        }
    }
}
