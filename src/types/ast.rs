//! AST representing a template.

use crate::types::span::Span;

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Template {
    pub scope: Scope,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Scope {
    pub stmts: Vec<Stmt>,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub enum Stmt {
    Raw(Span),
    InlineExpr(InlineExpr),
    Include(Include),
    IfElse(IfElse),
    ForLoop(ForLoop),
    With(With),
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct InlineExpr {
    pub expr: Expr,
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Include {
    pub name: String,
    pub globals: Option<Expr>,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct String {
    pub name: std::string::String,
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct IfElse {
    pub not: bool,
    pub cond: Expr,
    pub then_branch: Scope,
    pub else_branch: Option<Scope>,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct ForLoop {
    pub vars: LoopVars,
    pub iterable: Expr,
    pub body: Scope,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub enum LoopVars {
    Item(Ident),
    KeyValue(KeyValue),
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct KeyValue {
    pub key: Ident,
    pub value: Ident,
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct With {
    pub expr: Expr,
    pub name: Ident,
    pub body: Scope,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub enum Expr {
    Base(BaseExpr),
    Call(Call),
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Call {
    pub name: Ident,
    pub args: Option<Args>,
    pub receiver: Box<Expr>,
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Args {
    pub values: Vec<BaseExpr>,
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub enum BaseExpr {
    Var(Var),
    Literal(Literal),
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Var {
    pub path: Vec<Key>,
}

#[derive(Clone, Copy)]
#[cfg_attr(internal_debug, derive(Debug))]
pub enum Key {
    List(Index),
    Map(Ident),
}

#[derive(Clone, Copy)]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct Index {
    pub value: usize,
    pub span: Span,
}

#[derive(Clone, Copy)]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct Ident {
    pub span: Span,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Literal {
    pub value: crate::Value,
    pub span: Span,
}

impl Scope {
    pub const fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Base(base) => base.span(),
            Self::Call(call) => call.span,
        }
    }
}

impl BaseExpr {
    pub fn span(&self) -> Span {
        match self {
            BaseExpr::Var(var) => var.span(),
            BaseExpr::Literal(lit) => lit.span,
        }
    }
}

impl Var {
    pub fn span(&self) -> Span {
        self.first().span().combine(self.last().span())
    }

    pub fn first(&self) -> &Key {
        self.path.first().unwrap()
    }

    pub fn last(&self) -> &Key {
        self.path.last().unwrap()
    }

    pub fn rest(&self) -> &[Key] {
        &self.path[1..]
    }
}

impl Key {
    pub const fn span(&self) -> Span {
        match self {
            Key::List(key) => key.span,
            Key::Map(key) => key.span,
        }
    }
}
