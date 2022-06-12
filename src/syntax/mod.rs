mod ahocorasick;

use crate::syntax::ahocorasick::AhoCorasick;

#[derive(Debug)]
pub struct Searcher {
    imp: AhoCorasick,
}

#[derive(Debug)]
pub struct Syntax<'a> {
    pub begin_expr: &'a str,
    pub end_expr: &'a str,
    pub begin_block: &'a str,
    pub end_block: &'a str,
}

pub enum Kind {
    /// Begin expression delimiter, e.g. `{{`
    BeginExpr,
    /// End expression delimiter, e.g. `}}`
    EndExpr,
    /// Begin block delimiter, e.g. `{%`
    BeginBlock,
    /// End block delimiter, e.g. `%}`
    EndBlock,
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher {
    pub fn new() -> Self {
        Self::with_syntax(Syntax::new())
    }

    pub fn with_syntax(syntax: Syntax) -> Self {
        let imp = AhoCorasick::new([
            syntax.begin_expr,
            syntax.end_expr,
            syntax.begin_block,
            syntax.end_block,
        ]);
        Self { imp }
    }

    pub fn find_at<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize, usize)>
    where
        T: AsRef<[u8]>,
    {
        self.imp.find_at(haystack, at).map(|m| {
            let kind = to_kind(m);
            (kind, m.start(), m.end())
        })
    }

    pub fn starts_with<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize)>
    where
        T: AsRef<[u8]>,
    {
        let (kind, i, j) = self.find_at(haystack, at)?;
        (at == i).then(|| (kind, j))
    }
}

fn to_kind(m: ahocorasick::Match) -> Kind {
    match m.pattern_id() {
        0 => Kind::BeginExpr,
        1 => Kind::EndExpr,
        2 => Kind::BeginBlock,
        3 => Kind::EndBlock,
        _ => unreachable!(),
    }
}

impl Default for Syntax<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Syntax<'_> {
    pub const fn new() -> Self {
        Self {
            begin_expr: "{{",
            end_expr: "}}",
            begin_block: "{%",
            end_block: "%}",
        }
    }
}
