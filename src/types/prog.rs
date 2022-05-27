//! Defines a compiled [`Template`] which is a sequence of [`Instr`] that can be
//! executed by the renderer.

use crate::types::ast;
use crate::types::span::Span;

pub const FIXME: usize = !0;

#[derive(Debug)]
pub struct Template<'source> {
    pub source: &'source str,
    pub instrs: Vec<Instr<'source>>,
}

#[derive(Debug)]
pub enum Instr<'source> {
    /// Emit raw template
    EmitRaw(&'source str),

    /// Start a loop over value items
    StartLoop(ast::LoopVars<'source>, Span),

    /// Iterate the loop on the stack
    Iterate(usize),

    /// Jump to an instruction
    Jump(usize),

    /// Jump to the instruction if the value is false
    JumpIfFalse(usize, Span),

    /// Lookup a variable and push it onto the stack
    Push(ast::Path<'source>),

    /// Pop and emit the value at the top of the stack
    PopEmit(Span),

    /// Apply the function to the value at the top of the stack
    Call(ast::Ident<'source>),
}
