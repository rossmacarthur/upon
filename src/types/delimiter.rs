#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    BeginExpr,
    EndExpr,
    BeginExprTrim,
    EndExprTrim,
    BeginBlock,
    EndBlock,
    BeginBlockTrim,
    EndBlockTrim,
    BeginComment,
    EndComment,
    BeginCommentTrim,
    EndCommentTrim,
}
