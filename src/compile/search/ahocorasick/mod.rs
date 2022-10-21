//! A fast, multi-pattern searcher based on [Aho-Corasick algorithm][wikipedia].
//!
//! The design presented here mostly implements the standard algorithm as well
//! as some unique ideas from the excellent [`aho-corasick`][aho-corasick]
//! crate. This implementation only supports non-overlapping, leftmost-longest
//! match first semantics.
//!
//! [aho-corasick]: https://crates.io/crates/aho-corasick
//! [wikipedia]: https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm

mod build;
mod state;

use self::build::Builder;
use self::state::{State, DEAD, FAIL, S, START};

#[cfg_attr(internal_debug, derive(Debug))]
pub struct AhoCorasick {
    states: Vec<State>,
}

#[cfg_attr(internal_debug, derive(Debug))]
pub struct Match {
    pattern: Pattern,
    end: usize,
}

#[derive(Clone, Copy)]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct Pattern {
    id: usize,
    len: usize,
}

impl AhoCorasick {
    pub fn new<I, X, P>(patterns: I) -> Self
    where
        I: IntoIterator<Item = (X, P)>,
        X: Into<usize>,
        P: AsRef<[u8]>,
    {
        Builder::default().build(patterns)
    }

    pub fn find_at<T>(&self, haystack: T, mut at: usize) -> Option<Match>
    where
        T: AsRef<[u8]>,
    {
        let haystack = haystack.as_ref();

        let mut state = START;
        let mut last_match = self.get_match(state, 0, at);
        while at < haystack.len() {
            state = self.next_state(state, haystack[at]);
            debug_assert!(
                state != FAIL,
                "an automaton should never return fail state for next state"
            );
            at += 1;

            if state == DEAD {
                debug_assert!(
                    last_match.is_some(),
                    "an automaton should never return a dead state without a prior match"
                );
                return last_match;
            }

            if let Some(m) = self.get_match(state, 0, at) {
                last_match = Some(m);
            }
        }
        last_match
    }

    fn get_match(&self, id: S, match_id: usize, end: usize) -> Option<Match> {
        self.state(id)
            .matches
            .get(match_id)
            .map(|&pattern| Match { pattern, end })
    }

    fn next_state(&self, mut id: S, byte: u8) -> S {
        loop {
            let state = self.state(id);
            let next = state.next_state(byte);
            if next != FAIL {
                return next;
            }
            id = state.fail;
        }
    }

    fn state(&self, id: S) -> &State {
        &self.states[id]
    }
}

impl Match {
    pub fn pattern_id(&self) -> usize {
        self.pattern.id
    }

    /// The starting position of the match.
    pub fn start(&self) -> usize {
        self.end - self.pattern.len
    }

    /// The ending position of the match.
    pub fn end(&self) -> usize {
        self.end
    }
}

impl Pattern {
    fn new(id: usize, len: usize) -> Self {
        Self { id, len }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aho_corasick_basics() {
        t(&[], "", &[]);
        t(&["a"], "", &[]);
        t(&["a"], "a", &[(0, 0, 1)]);
        t(&["a"], "aa", &[(0, 0, 1), (0, 1, 2)]);
        t(&["a"], "aaa", &[(0, 0, 1), (0, 1, 2), (0, 2, 3)]);
        t(&["a"], "aba", &[(0, 0, 1), (0, 2, 3)]);
        t(&["a"], "bba", &[(0, 2, 3)]);
        t(&["a"], "bbb", &[]);
        t(&["a"], "bababbbba", &[(0, 1, 2), (0, 3, 4), (0, 8, 9)]);
        t(&["aa"], "", &[]);
        t(&["aa"], "aa", &[(0, 0, 2)]);
        t(&["aa"], "aabbaa", &[(0, 0, 2), (0, 4, 6)]);
        t(&["aa"], "abbab", &[]);
        t(&["aa"], "abbabaa", &[(0, 5, 7)]);
        t(&["abc"], "abc", &[(0, 0, 3)]);
        t(&["abc"], "zazabzabcz", &[(0, 6, 9)]);
        t(&["abc"], "zazabczabcz", &[(0, 3, 6), (0, 7, 10)]);
        t(&["a", "b"], "", &[]);
        t(&["a", "b"], "z", &[]);
        t(&["a", "b"], "b", &[(1, 0, 1)]);
        t(&["a", "b"], "a", &[(0, 0, 1)]);
        t(
            &["a", "b"],
            "abba",
            &[(0, 0, 1), (1, 1, 2), (1, 2, 3), (0, 3, 4)],
        );
        t(
            &["b", "a"],
            "abba",
            &[(1, 0, 1), (0, 1, 2), (0, 2, 3), (1, 3, 4)],
        );
        t(&["abc", "bc"], "xbc", &[(1, 1, 3)]);
        t(&["foo", "bar"], "", &[]);
        t(&["foo", "bar"], "foobar", &[(0, 0, 3), (1, 3, 6)]);
        t(&["foo", "bar"], "barfoo", &[(1, 0, 3), (0, 3, 6)]);
        t(&["foo", "bar"], "foofoo", &[(0, 0, 3), (0, 3, 6)]);
        t(&["foo", "bar"], "barbar", &[(1, 0, 3), (1, 3, 6)]);
        t(&["foo", "bar"], "bafofoo", &[(0, 4, 7)]);
        t(&["bar", "foo"], "bafofoo", &[(1, 4, 7)]);
        t(&["foo", "bar"], "fobabar", &[(1, 4, 7)]);
        t(&["bar", "foo"], "fobabar", &[(0, 4, 7)]);
        t(&[""], "", &[(0, 0, 0)]);
        t(&[""], "a", &[(0, 0, 0), (0, 1, 1)]);
        t(&[""], "abc", &[(0, 0, 0), (0, 1, 1), (0, 2, 2), (0, 3, 3)]);
        t(&["yabcdef", "abcdezghi"], "yabcdefghi", &[(0, 0, 7)]);
        t(&["yabcdef", "abcdezghi"], "yabcdezghi", &[(1, 1, 10)]);
        t(
            &["yabcdef", "bcdeyabc", "abcdezghi"],
            "yabcdezghi",
            &[(2, 1, 10)],
        );
    }

    #[test]
    fn aho_corasick_non_overlapping() {
        t(&["abcd", "bcd", "cd"], "abcd", &[(0, 0, 4)]);
        t(&["bcd", "cd", "abcd"], "abcd", &[(2, 0, 4)]);
        t(&["abc", "bc"], "zazabcz", &[(0, 3, 6)]);
        t(&["ab", "ba"], "abababa", &[(0, 0, 2), (0, 2, 4), (0, 4, 6)]);
        t(&["foo", "foo"], "foobarfoo", &[(0, 0, 3), (0, 6, 9)]);
        t(&["", ""], "", &[(0, 0, 0)]);
        t(&["", ""], "a", &[(0, 0, 0), (0, 1, 1)]);
    }

    #[test]
    fn aho_corasick_leftmost() {
        t(&["ab", "ab"], "abcd", &[(0, 0, 2)]);
        t(&["a", ""], "a", &[(0, 0, 1), (1, 1, 1)]);
        t(&["", ""], "a", &[(0, 0, 0), (0, 1, 1)]);
        t(&["a", "ab"], "aa", &[(0, 0, 1), (0, 1, 2)]);
        t(&["ab", "a"], "aa", &[(1, 0, 1), (1, 1, 2)]);
        t(&["ab", "a"], "xayabbbz", &[(1, 1, 2), (0, 3, 5)]);
        t(&["abcd", "bce", "b"], "abce", &[(1, 1, 4)]);
        t(&["abcd", "ce", "bc"], "abce", &[(2, 1, 3)]);
        t(&["abcd", "bce", "ce", "b"], "abce", &[(1, 1, 4)]);
        t(&["abcd", "bce", "cz", "bc"], "abcz", &[(3, 1, 3)]);
        t(&["bce", "cz", "bc"], "bcz", &[(2, 0, 2)]);
        t(&["abc", "bd", "ab"], "abd", &[(2, 0, 2)]);
        t(&["abcdefghi", "hz", "abcdefgh"], "abcdefghz", &[(2, 0, 8)]);
        t(
            &["abcdefghi", "cde", "hz", "abcdefgh"],
            "abcdefghz",
            &[(3, 0, 8)],
        );
        t(
            &["abcdefghi", "hz", "abcdefgh", "a"],
            "abcdefghz",
            &[(2, 0, 8)],
        );
        t(
            &["b", "abcdefghi", "hz", "abcdefgh"],
            "abcdefghz",
            &[(3, 0, 8)],
        );
        t(
            &["h", "abcdefghi", "hz", "abcdefgh"],
            "abcdefghz",
            &[(3, 0, 8)],
        );
        t(
            &["z", "abcdefghi", "hz", "abcdefgh"],
            "abcdefghz",
            &[(3, 0, 8), (0, 8, 9)],
        );
    }

    #[test]
    fn aho_corasick_leftmost_longest() {
        t(&["ab", "abcd"], "abcd", &[(1, 0, 4)]);
        t(&["abcd", "bcd", "cd", "b"], "abcd", &[(0, 0, 4)]);
        t(&["", "a"], "a", &[(1, 0, 1), (0, 1, 1)]);
        t(&["", "a", ""], "a", &[(1, 0, 1), (0, 1, 1)]);
        t(&["a", "", ""], "a", &[(0, 0, 1), (1, 1, 1)]);
        t(&["", "", "a"], "a", &[(2, 0, 1), (0, 1, 1)]);
        t(&["", "a"], "aa", &[(1, 0, 1), (1, 1, 2), (0, 2, 2)]);
        t(&["a", "ab"], "a", &[(0, 0, 1)]);
        t(&["a", "ab"], "ab", &[(1, 0, 2)]);
        t(&["ab", "a"], "a", &[(1, 0, 1)]);
        t(&["ab", "a"], "ab", &[(0, 0, 2)]);
        t(&["abcdefg", "bcde", "bcdef"], "abcdef", &[(2, 1, 6)]);
        t(&["abcdefg", "bcdef", "bcde"], "abcdef", &[(1, 1, 6)]);
        t(&["abcd", "b", "bce"], "abce", &[(2, 1, 4)]);
        t(
            &["a", "abcdefghi", "hz", "abcdefgh"],
            "abcdefghz",
            &[(3, 0, 8)],
        );
        t(&["a", "abab"], "abab", &[(1, 0, 4)]);
        t(&["abcd", "b", "ce"], "abce", &[(1, 1, 2), (2, 2, 4)]);
        t(&["a", "ab"], "xayabbbz", &[(0, 1, 2), (1, 3, 5)]);
    }

    #[track_caller]
    fn t(patterns: &[&str], haystack: &str, exp: &[(usize, usize, usize)]) {
        let ac = AhoCorasick::new(patterns.iter().enumerate());
        let matches: Vec<_> = ac
            .find_iter(haystack.as_ref())
            .map(|m| (m.pattern_id(), m.start(), m.end()))
            .take(10)
            .collect();
        assert_eq!(matches, exp);
    }

    impl AhoCorasick {
        pub fn find_iter<'a>(&'a self, haystack: &'a [u8]) -> impl Iterator<Item = Match> + 'a {
            let mut pos = 0;
            std::iter::from_fn(move || {
                if pos > haystack.len() {
                    return None;
                }
                let mat = self.find_at(haystack, pos)?;
                if mat.end() == pos {
                    // If the automaton can match the empty string and if we
                    // found an empty match, then we need to forcefully move the
                    // position.
                    pos += 1;
                } else {
                    pos = mat.end();
                }

                Some(mat)
            })
        }
    }
}
