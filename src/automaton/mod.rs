pub use pattern::{Pattern};
use std::fmt;
use std::hash::Hash;
use std::iter;
use std::mem;
use std::ops::Range;
use std::ops::{Index, IndexMut};

#[macro_use]
mod pattern;
pub mod converter;
pub mod dfa;
pub mod nfa;

pub use dfa::DFA;
pub use nfa::NFA;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ByteClassId(u16);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct ByteClass(Vec<u8>);

impl ByteClass {
    pub(crate) fn empty() -> Self {
        ByteClass(vec![0; 256])
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.iter().all(|t| *t == 0)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ByteSet([bool; 256]);

impl ByteSet {
    fn empty() -> Self {
        Self([false; 256])
    }

    pub fn not(&mut self) {}

    pub fn add_range(&mut self, range: Range<u8>) {
        let min = range.start;
        let max = range.end;
        self.0[min as usize] = true;
        if max < 255 {
            self.0[max as usize + 1] = true;
        }
    }
}

impl From<ByteSet> for ByteClass {
    fn from(set: ByteSet) -> Self {
        Self(
            set.0
                .to_vec()
                .into_iter()
                .scan(0u8, |acc, x| Some(*acc + (x as u8)))
                .collect(),
        )
    }
}

impl From<Range<u8>> for ByteSet {
    fn from(range: Range<u8>) -> Self {
        let mut set = ByteSet::empty();
        set.add_range(range);
        set
    }
}

impl From<Range<u8>> for ByteClass {
    fn from(range: Range<u8>) -> Self {
        Self::from(ByteSet::from(range))
    }
}

impl fmt::Debug for ByteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.0.to_vec())?;
        Ok(())
    }
}

impl Index<u8> for ByteClass {
    type Output = u8;
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for ByteClass {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
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
