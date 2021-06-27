use super::byteclass::ByteClassId;
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(pub u32);

impl StateId {
    pub fn of(id: usize) -> Self {
        assert!(id < u32::MAX as usize);
        Self(id as u32)
    }

    pub(crate) fn add(&self, n: usize) -> Self {
        Self::of(self.0 as usize + n)
    }
}

#[derive(Debug, Clone)]
pub struct NfaState {
    /*
        If the byteclass at possition 5 equals to 2, then there is a edge
        in the nfa going from self to every state in self.table[2]. The
        byte that this edge contains is 5.
    */
    pub(crate) table: Vec<StateId>,
    /*
        The byteclassid 0 is always a refference to a byteclass containig
        only zeros.
    */
    pub(crate) class: ByteClassId,

    /*
        The epsilon/lambda transitions going from this state.
    */
    pub(crate) epsilons: Vec<StateId>,

    /*
        We use assosiated values to keep track of what command
        is assosiated with the given NfaState.
    */
    pub(crate) assosiations: bit_set::BitSet,
}

impl Default for NfaState {
    fn default() -> Self {
        Self {
            table: vec![],
            class: Default::default(),
            epsilons: Default::default(),
            assosiations: Default::default(),
        }
    }
}

impl NfaState {
    pub(crate) fn push_epsilon(&mut self, to: StateId) {
        if !self.epsilons.contains(&to) {
            self.epsilons.push(to);
        }
    }
}

// Returns the states one would get to from self if the input u8 was 'index'.
impl Index<u8> for NfaState {
    type Output = StateId;
    fn index(&self, index: u8) -> &Self::Output {
        &self.table[index as usize]
    }
}
