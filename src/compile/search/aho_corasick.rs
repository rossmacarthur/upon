use crate::types::delimiter::Delimiter;
use crate::types::syntax::Syntax;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct AhoCorasickSearcher {
    imp: AhoCorasick,
    delimiters: Vec<Delimiter>,
}

impl AhoCorasickSearcher {
    pub fn new(syntax: Syntax) -> Self {
        let imp = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .build(syntax.patterns)
            .expect("failed to build AhoCorasick");
        Self {
            imp,
            delimiters: syntax.delimiters,
        }
    }

    #[inline]
    pub fn find_at(&self, source: &str, at: usize) -> Option<(Delimiter, usize, usize)> {
        let sb = source.as_bytes();
        self.imp.find(&sb[at..]).map(|m| {
            let delimiter = self.delimiters[m.pattern()];
            (delimiter, at + m.start(), at + m.end())
        })
    }

    #[inline]
    pub fn starts_with(&self, source: &str, at: usize) -> Option<(Delimiter, usize)> {
        let (delimiter, i, j) = self.find_at(source, at)?;
        if at == i {
            Some((delimiter, j))
        } else {
            None
        }
    }
}
