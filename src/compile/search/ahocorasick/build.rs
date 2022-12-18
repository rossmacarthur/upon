//! A builder for an Aho-Corasick automaton.
//!
//! From the given set of patterns we build a state machine with a series of
//! states that encode a transition for every possible byte. This state machine
//! can then used to simultaneously search a string for the patterns.
//!
//! Consider building an Aho-Corasick automaton with the following patterns:
//! 'ab' and 'cd', the trie would look the following. Where the states are
//! represented as `S?` and have an asterisk (`*`) if there any matches at that
//! state.
//!
//! ```text
//!      a - S1 - b - S2*
//!     /
//! S0 - c - S3 - d - S4*
//! ```
//!
//! In the above state machine there are no bytes that are the same between the
//! patterns. Now consider the following patterns: 'abe' and 'bcd'. In the case
//! of an input text of 'abcd', when at S2 we would end up failing to transition
//! to S3. But we can encode the failure in the automaton as a transition from
//! S2 to S4 and continue the search. What is not shown in these diagrams is that
//! *all* states have a failure transition, but only S2 has a *non-trivial*
//! failure transition. That is, all other states have a failure transition back
//! to the start state.
//!
//! ```text
//!      a - S1 - b - S2 - e - S3*
//!     /             /
//!    /       -------
//!   /       /
//! S0 - b - S4 - c - S5 - d - S6*
//! ```
//!
//! Encoding the failure transitions is the most complex part of building the
//! automaton. Traditionally, this is implemented using a breadth-first search
//! starting with all transitions from the start state. For each state and for
//! every input transition at that state we follow the failure transitions
//! backward until we find a failure state that has a forward transition for
//! that input. That state must be the fail state for the original state.
//!
//! In order to support leftmost-longest match first semantics we also need
//! to make a few modifications to the way the failure transitions are built.

use std::collections::VecDeque;

use super::{AhoCorasick, Pattern, State, DEAD, FAIL, S, START};

#[derive(Default)]
pub struct Builder {
    states: Vec<State>,
}

impl Builder {
    pub fn build<I, X, P>(mut self, patterns: I) -> AhoCorasick
    where
        I: IntoIterator<Item = (X, P)>,
        X: Into<usize>,
        P: AsRef<[u8]>,
    {
        self.push_state(0); // the fail state
        self.push_state(0); // the dead state
        self.push_state(0); // the start state
        self.build_initial_trie(patterns);

        // Set the failure transitions in the start state to loop back to the
        // start state.
        let start = self.start_mut();
        for byte in all() {
            if start.next_state(byte) == FAIL {
                start.set_transition(byte, START);
            }
        }

        // Set the failure transitions in the dead state to loop back to the
        // dead state.
        let dead = self.state_mut(DEAD);
        for byte in all() {
            if dead.next_state(byte) == FAIL {
                dead.set_transition(byte, DEAD);
            }
        }

        self.fill_failure_transitions();

        // Remove the start state loop by rewriting any transitions on the start
        // state back to the start state with transitions to the dead state.
        if self.start().is_match() {
            let start = self.start_mut();
            for byte in all() {
                if start.next_state(byte) == START {
                    start.set_transition(byte, DEAD);
                }
            }
        }

        let Self { states } = self;
        AhoCorasick { states }
    }

    /// Build the initial trie where each pattern has a path from the start
    /// state until the end of the pattern.
    fn build_initial_trie<I, X, P>(&mut self, patterns: I)
    where
        I: IntoIterator<Item = (X, P)>,
        X: Into<usize>,
        P: AsRef<[u8]>,
    {
        for (pattern_id, pattern) in patterns.into_iter() {
            let pattern = pattern.as_ref();

            let mut id = START;
            for (depth, &byte) in pattern.iter().enumerate() {
                let next = self.state(id).next_state(byte);
                if next == FAIL {
                    let next = self.push_state(depth + 1);
                    self.state_mut(id).set_transition(byte, next);
                    id = next;
                } else {
                    id = next;
                }
            }

            let p = Pattern::new(pattern_id.into(), pattern.len());
            self.state_mut(id).push_match(p);
        }
    }

    fn fill_failure_transitions(&mut self) {
        // Initialize the queue for breadth first search with all transitions
        // out of the start state. We handle the start state specially because
        // we only want to follow non-self transitions. If we followed self
        // transitions, then this would never terminate.
        let mut queue = VecDeque::new();
        for byte in all() {
            let next = self.start().next_state(byte);
            if next != START {
                let match_depth = if self.start().is_match() {
                    Some(0)
                } else {
                    None
                };
                queue.push_back((next, match_depth));

                // If a state immediately following the start state is a match
                // state, then we never want to follow its failure transition
                // since the failure transition necessarily leads back to the
                // start state, which we never want to do for leftmost matching
                // after a match has been found.
                //
                // N.B. This is a special case of the more general handling
                // found below.
                if self.state(next).is_match() {
                    self.state_mut(next).fail = DEAD;
                }
            }
        }

        while let Some((curr, match_depth)) = queue.pop_front() {
            let prev_len = queue.len();

            for byte in all() {
                let next = self.state(curr).next_state(byte);
                if next == FAIL {
                    continue;
                }

                let next_match_depth = match match_depth {
                    Some(d) => Some(d),
                    None if self.state(next).is_match() => {
                        let depth = self.state(next).depth
                            - self.state(next).get_longest_match_len().unwrap()
                            + 1;
                        Some(depth)
                    }
                    None => None,
                };

                queue.push_back((next, next_match_depth));

                let fail = {
                    let mut id = self.state(curr).fail;
                    while self.state(id).next_state(byte) == FAIL {
                        id = self.state(id).fail;
                    }
                    self.state(id).next_state(byte)
                };

                // Thanks Andrew Gallant
                if let Some(match_depth) = next_match_depth {
                    let fail_depth = self.state(fail).depth;
                    let next_depth = self.state(next).depth;
                    if next_depth - match_depth + 1 > fail_depth {
                        self.state_mut(next).fail = DEAD;
                        continue;
                    }
                    assert_ne!(
                        self.state(next).fail,
                        START,
                        "states that are match states or follow match \
                         states should never have a failure transition \
                         back to the start state in leftmost searching",
                    );
                }

                self.state_mut(next).fail = fail;
                self.copy_matches(fail, next);
            }

            // If there are no transitions for this state and if it's a match
            // state, then we must set its failure transition to the dead
            // state since we never want it to restart the search.
            if queue.len() == prev_len && self.state(curr).is_match() {
                self.state_mut(curr).fail = DEAD;
            }

            // We don't need to copy empty matches from the start state here
            // because that's only necessary for overlapping matches and
            // leftmost match kinds don't support overlapping matches.
        }
    }

    fn copy_matches(&mut self, src: S, dst: S) {
        assert!(src != dst, "src {src} must not be equal to dst {dst}");

        // Simply gets a mutable reference to both states.
        let i = src;
        let j = dst;
        let (src, dst) = if i < j {
            let (left, right) = self.states.split_at_mut(j);
            (&mut left[i], &mut right[0])
        } else {
            let (left, right) = self.states.split_at_mut(i);
            (&mut right[0], &mut left[j])
        };

        dst.matches.extend_from_slice(&src.matches);
    }

    fn push_state(&mut self, depth: usize) -> S {
        let id = self.states.len();
        self.states.push(State {
            depth,
            fail: START,
            trans: [FAIL; 256],
            matches: vec![],
        });
        id
        // match id.try_into() {
        //     Ok(id) => id,
        //     Err(_) => {
        //         panic!(
        //             "state id type `{}` too small for the \
        //              number of states in the automaton",
        //             std::any::type_name::<S>()
        //         );
        //     }
        // }
    }

    fn state(&self, id: S) -> &State {
        &self.states[id]
    }

    fn state_mut(&mut self, id: S) -> &mut State {
        &mut self.states[id]
    }

    fn start(&self) -> &State {
        self.state(START)
    }

    fn start_mut(&mut self) -> &mut State {
        self.state_mut(START)
    }
}

impl State {
    fn push_match(&mut self, p: Pattern) {
        self.matches.push(p);
    }

    fn set_transition(&mut self, byte: u8, to: S) {
        self.trans[byte as usize] = to;
    }

    fn get_longest_match_len(&self) -> Option<usize> {
        // Why is this true? Because the first match in any matching state
        // will always correspond to the match added to it during trie
        // construction (since when we copy matches due to failure transitions,
        // we always append them). Therefore, it follows that the first match
        // must always be longest since any subsequent match must be from a
        // failure transition, and a failure transition by construction points
        // to a proper suffix. A proper suffix is, by definition, smaller.
        self.matches.get(0).map(|&p| p.len)
    }
}

fn all() -> impl Iterator<Item = u8> {
    0..=255
}
