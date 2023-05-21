//! Defines a compiled [`Template`] which is a sequence of [`Instr`] that can be
//! executed by the renderer.

use std::borrow::Cow;

use crate::types::ast;
use crate::types::span::Span;
use crate::Value;

pub const FIXME: usize = !0;

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Template<'source> {
    pub source: Cow<'source, str>,
    pub instrs: Vec<Instr>,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub enum Instr {
    /// Jump to an instruction
    Jump(usize),

    /// Jump to the instruction if the current expression is true
    JumpIfTrue(usize),

    /// Jump to the instruction if the current expression is false
    JumpIfFalse(usize),

    /// Emit the current expression
    Emit(Span),

    /// Emit raw template
    EmitRaw(Span),

    /// Apply the filter or value formatter to the current expression and emit
    EmitWith(ast::Ident, Span),

    /// Start a loop over the current expression
    LoopStart(ast::LoopVars, Span),

    /// Advance and jump to the start of the loop
    LoopNext(usize),

    /// Push the current expression to the stack as a variable
    WithStart(ast::Ident),

    /// Remove a previously added variable from the stack
    WithEnd,

    /// Render a template
    Include(ast::String),

    /// Render a template with the current expression
    IncludeWith(ast::String),

    /// Lookup a variable and start building an expression
    ExprStart(ast::Var),

    /// Start building an expression using a literal
    ExprStartLit(Value),

    /// Apply the filter to the value at the top of the stack
    Apply(ast::Ident, Span, Option<ast::Args>),
}

#[cfg(not(internal_debug))]
impl std::fmt::Debug for Template<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<compiled>")
    }
}
