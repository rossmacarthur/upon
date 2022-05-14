use std::cmp::{max, min};
use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub m: usize,
    pub n: usize,
}

impl Span {
    pub fn new(m: usize, n: usize) -> Self {
        Self { m, n }
    }

    pub fn combine(self, other: Span) -> Self {
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
