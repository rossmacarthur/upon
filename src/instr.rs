use crate::ast;
use crate::span::Span;

pub const FIXME: usize = !0;

/// Represents a compiled template.
#[derive(Debug)]
pub struct Template<'source> {
    pub source: &'source str,
    pub instrs: Vec<Instr<'source>>,
    pub spans: Vec<Span>,
}

/// Represents an instruction in a template program.
#[derive(Debug)]
pub enum Instr<'source> {
    /// Emit raw template
    EmitRaw(&'source str),

    /// Start a loop over value items
    StartLoop(usize, ast::LoopVars<'source>, Span),

    /// Iterate the loop on the stack
    Iterate(usize),

    /// Jump to the instruction if the value is true
    JumpIfTrue(usize, Span),

    /// Jump to the instruction if the value is false
    JumpIfFalse(usize, Span),

    /// Lookup a variable and push it onto the stack
    Push(ast::Path<'source>),

    /// Pop and emit the value at the top of the stack
    PopEmit(Span),

    /// Apply the function to the value at the top of the stack
    Call(ast::Ident<'source>),
}
