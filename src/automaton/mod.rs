use crate::graph::NodeKind;
pub use pattern::Pattern;
use std::hash::Hash;
use std::iter;
use std::mem;
use std::ops::{Index, IndexMut};

#[macro_use]
pub mod pattern;
pub mod dfa;
pub mod nfa;
pub mod converter;

pub use nfa::NFA;
pub use dfa::DFA;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StateId(u32);

impl StateId {
    fn of(id: u32) -> Self {
        assert!(id < u32::MAX);
        StateId(id)
    }

    fn add(&self, n: u32) -> Self {
        StateId::of(self.0 + n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc() {
        let nfa = NFA::from(&pattern!("abc"));
        let dfa = DFA::from(nfa);
        assert!(dfa.find("").is_none());
        assert!(dfa.find("abc").is_some());
        assert!(dfa.find("abca").is_none());
    }

    #[test]
    fn abc_repeat() {
        let nfa = NFA::from(&pattern!("abc"*));
        let dfa = DFA::from(nfa);
        assert!(dfa.find("").is_some());
        assert!(dfa.find("abc").is_some());
        assert!(dfa.find("abcabc").is_some());
        assert!(dfa.find("abca").is_none());
    }

    #[test]
    fn minimize() {
        let nfa = NFA::from(&pattern!("abc"*));
        let dfa = DFA::from(nfa);
        let minimized_dfa = dfa.minimize();
    }
}