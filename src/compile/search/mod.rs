#[cfg(feature = "syntax")]
mod aho_corasick;

#[cfg(feature = "syntax")]
use crate::compile::search::aho_corasick::AhoCorasickSearcher;
use crate::types::delimiter::Delimiter;
#[cfg(feature = "syntax")]
use crate::types::syntax::Syntax;

#[cfg_attr(internal_debug, derive(Debug))]
pub enum Searcher {
    Default(DefaultSearcher),
    #[cfg(feature = "syntax")]
    AhoCorasick(AhoCorasickSearcher),
}

impl Searcher {
    pub fn new() -> Self {
        Self::Default(DefaultSearcher)
    }

    #[cfg(feature = "syntax")]
    pub fn with_syntax(syntax: Syntax) -> Self {
        Self::AhoCorasick(AhoCorasickSearcher::new(syntax))
    }

    #[inline]
    pub fn find_at(&self, source: &str, at: usize) -> Option<(Delimiter, usize, usize)> {
        match self {
            Self::Default(searcher) => searcher.find_at(source, at),
            #[cfg(feature = "syntax")]
            Self::AhoCorasick(searcher) => searcher.find_at(source, at),
        }
    }

    #[inline]
    pub fn starts_with(&self, source: &str, i: usize) -> Option<(Delimiter, usize)> {
        match self {
            Self::Default(searcher) => searcher.starts_with(source, i),
            #[cfg(feature = "syntax")]
            Self::AhoCorasick(searcher) => searcher.starts_with(source, i),
        }
    }
}

#[cfg(not(internal_debug))]
impl std::fmt::Debug for Searcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default(_) => f.debug_tuple("DefaultSearcher").finish(),
            #[cfg(feature = "syntax")]
            Self::AhoCorasick(_) => f
                .debug_struct("AhoCorasickSearcher")
                .finish_non_exhaustive(),
        }
    }
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct DefaultSearcher;

impl DefaultSearcher {
    #[inline]
    fn find_at(&self, source: &str, mut at: usize) -> Option<(Delimiter, usize, usize)> {
        let sb = source.as_bytes();
        loop {
            let mark = at + sb[at..].iter().position(|&b| b == b'{' || b == b'}')?;
            if sb[mark] == b'{' {
                let i = mark;
                match &sb[i..] {
                    // expr
                    [b'{', b'{', b'-', ..] => return Some((Delimiter::BeginExprTrim, i, i + 3)),
                    [b'{', b'{', ..] => return Some((Delimiter::BeginExpr, i, i + 2)),
                    // block
                    [b'{', b'%', b'-', ..] => return Some((Delimiter::BeginBlockTrim, i, i + 3)),
                    [b'{', b'%', ..] => return Some((Delimiter::BeginBlock, i, i + 2)),
                    // comment
                    [b'{', b'#', b'-', ..] => return Some((Delimiter::BeginCommentTrim, i, i + 3)),
                    [b'{', b'#', ..] => return Some((Delimiter::BeginComment, i, i + 2)),
                    _ => at = i + 1,
                }
            } else {
                let j = mark + 1;
                let i = j.saturating_sub(3);
                match &sb[i..] {
                    // expr
                    [b'-', b'}', b'}', ..] => return Some((Delimiter::EndExprTrim, i, i + 3)),
                    [_, b'}', b'}', ..] => return Some((Delimiter::EndExprTrim, i + 1, i + 3)),
                    [b'}', b'}', ..] => return Some((Delimiter::EndExpr, i, i + 2)),
                    // block
                    [b'-', b'%', b'}', ..] => return Some((Delimiter::EndBlockTrim, i, i + 3)),
                    [_, b'%', b'}', ..] => return Some((Delimiter::EndBlock, i + 1, i + 3)),
                    [b'%', b'}', ..] => return Some((Delimiter::EndBlock, i, i + 2)),
                    // comment
                    [b'-', b'#', b'}', ..] => return Some((Delimiter::EndCommentTrim, i, i + 3)),
                    [_, b'#', b'}', ..] => return Some((Delimiter::EndComment, i + 1, i + 3)),
                    [b'#', b'}', ..] => return Some((Delimiter::EndComment, i, i + 2)),
                    _ => at = j,
                }
            }
        }
    }

    #[inline]
    fn starts_with(&self, source: &str, i: usize) -> Option<(Delimiter, usize)> {
        let sb = source.as_bytes();
        match &sb[i..] {
            // begin
            [b'{', b'{', b'-', ..] => Some((Delimiter::BeginExprTrim, i + 3)),
            [b'{', b'{', ..] => Some((Delimiter::BeginExpr, i + 2)),
            [b'{', b'%', b'-', ..] => Some((Delimiter::BeginBlockTrim, i + 3)),
            [b'{', b'%', ..] => Some((Delimiter::BeginBlock, i + 2)),
            [b'{', b'#', b'-', ..] => Some((Delimiter::BeginCommentTrim, i + 3)),
            [b'{', b'#', ..] => Some((Delimiter::BeginComment, i + 2)),
            // end
            [b'-', b'}', b'}', ..] => Some((Delimiter::EndExprTrim, i + 3)),
            [b'}', b'}', ..] => Some((Delimiter::EndExpr, i + 2)),
            [b'-', b'%', b'}', ..] => Some((Delimiter::EndBlockTrim, i + 3)),
            [b'%', b'}', ..] => Some((Delimiter::EndBlock, i + 2)),
            [b'-', b'#', b'}', ..] => Some((Delimiter::EndCommentTrim, i + 3)),
            [b'#', b'}', ..] => Some((Delimiter::EndComment, i + 2)),
            _ => None,
        }
    }
}
