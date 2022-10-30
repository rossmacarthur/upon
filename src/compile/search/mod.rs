mod ahocorasick;

use crate::compile::search::ahocorasick::AhoCorasick;
use crate::types::syntax::{Kind, Syntax};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Searcher {
    imp: AhoCorasick,
}

impl Searcher {
    pub fn new(syntax: Syntax) -> Self {
        let imp = AhoCorasick::new(syntax.patterns);
        Self { imp }
    }

    pub fn find_at<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize, usize)>
    where
        T: AsRef<[u8]>,
    {
        self.imp.find_at(haystack, at).map(|m| {
            let kind = Kind::from_usize(m.pattern_id());
            (kind, m.start(), m.end())
        })
    }

    pub fn starts_with<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize)>
    where
        T: AsRef<[u8]>,
    {
        let (kind, i, j) = self.find_at(haystack, at)?;
        if at == i {
            Some((kind, j))
        } else {
            None
        }
    }
}
