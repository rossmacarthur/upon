use super::Pattern;

/// A unique identifier for a state.
pub type S = usize;

/// The identifier for an automaton's fail state.
pub const FAIL: S = 0;

/// The identifier for an automaton's dead state.
pub const DEAD: S = 1;

/// The identifier for an automaton's start state.
pub const START: S = 2;

/// A state in an Aho-Corasick automaton.
#[cfg_attr(internal_debug, derive(Debug))]
pub struct State {
    /// The transitions to the next state.
    pub trans: [S; 256],

    /// The failure transition.
    pub fail: S,

    /// The patterns that are matched at this state.
    pub matches: Vec<Pattern>,

    /// The distance from the start state in the automaton.
    pub depth: usize,
}

impl State {
    /// Returns the next state for the given input byte.
    pub fn next_state(&self, byte: u8) -> S {
        self.trans[byte as usize]
    }

    /// Whether or not this state contains any matches.
    pub fn is_match(&self) -> bool {
        !self.matches.is_empty()
    }
}
