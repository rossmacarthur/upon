//! Defines a [`Span`] which is used to represent a region in the template
//! source code.

use std::cmp::{max, min};
use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub m: usize,
    pub n: usize,
}

impl Span {
    pub fn combine(self, other: Self) -> Self {
        let m = min(self.m, other.m);
        let n = max(self.n, other.n);
        Self { m, n }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, span: Span) -> &Self::Output {
        let Span { m, n } = span;
        &self[m..n]
    }
}

impl From<Range<usize>> for Span {
    fn from(r: Range<usize>) -> Self {
        Self {
            m: r.start,
            n: r.end,
        }
    }
}
