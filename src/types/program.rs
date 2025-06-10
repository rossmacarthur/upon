//! Defines a compiled [`Template`] which is a sequence of [`Instr`] that can be
//! executed by the renderer.

use std::borrow::Cow;

use crate::types::ast;
use crate::types::span::Span;

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
    Emit,

    /// Emit raw template
    EmitRaw(Span),

    /// Apply the formatter or function to the current expression and emit.
    ///
    /// The second value is the number of arguments to pop from the stack
    /// excluding the value itself.
    EmitWith(ast::Ident, usize, Span),

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
    ExprStartVar(ast::Var),

    /// Start building an expression using a literal
    ExprStartLiteral(ast::Literal),

    /// Start building a list expression
    ExprStartList(Span),

    /// Start building a map expression
    ExprStartMap(Span),

    /// Append an item to the current list expression
    ExprListPush,

    /// Insert an item to the current map expression
    ExprMapInsert(ast::String),

    /// Apply the function using the value and args on the top of the stack.
    ///
    /// The second value is the number of arguments to pop from the stack
    /// excluding the value itself.
    Apply(ast::Ident, usize, Span),
}

#[cfg(not(internal_debug))]
impl std::fmt::Debug for Template<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template").finish_non_exhaustive()
    }
}
