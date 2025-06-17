use crate::types::delimiter::Delimiter;
use crate::types::syntax::Syntax;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, Anchored, Input, MatchKind, StartKind};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct AhoCorasickSearcher {
    imp: AhoCorasick,
    delimiters: Vec<Delimiter>,
}

impl AhoCorasickSearcher {
    pub fn new(syntax: Syntax) -> Self {
        let imp = AhoCorasickBuilder::new()
            .start_kind(StartKind::Both)
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
        self.imp.find(Input::new(source).range(at..)).map(|m| {
            let delimiter = self.delimiters[m.pattern()];
            (delimiter, m.start(), m.end())
        })
    }

    #[inline]
    pub fn starts_with(&self, source: &str, at: usize) -> Option<(Delimiter, usize)> {
        self.imp
            .find(Input::new(source).range(at..).anchored(Anchored::Yes))
            .map(|m| {
                let delimiter = self.delimiters[m.pattern()];
                (delimiter, m.end())
            })
    }
}
