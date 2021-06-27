/*
A byteclass is just a [0u8; 256].  It is used by the nfa and encodes the edges/transitions going
from one state/node to another, and what u8 is assosiated with that edge.

Inside the nfa state structs we have a self.class:ByteclassId field that is a
refference to a byteclass. For the nfa there is a field self.table: Vec<Vec<StateId>>,

For the NfaState, if the byteclass has a 5 at possition 42, that means that if the input is
42 at the given state, we transition into self.table[5].
*/

use super::AutomataBuildError;
use std::{
    convert::{TryFrom, TryInto},
    ops::{Index, IndexMut},
    slice::Iter,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct ByteClassId(pub u32);

impl Default for ByteClassId {
    fn default() -> Self {
        ByteClassId(0)
    }
}

impl TryFrom<usize> for ByteClassId {
    type Error = AutomataBuildError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(x) => Ok(ByteClassId(x)),
            Err(_) => Err(AutomataBuildError::RanOutOfByteClassIds),
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct ByteClass([u8; 256]);

impl ByteClass {
    pub(crate) fn zeros() -> Self {
        ByteClass([0; 256])
    }
}

impl From<u8> for ByteClass {
    fn from(value: u8) -> Self {
        let mut values = [0u8; 256];
        values[value as usize] = 1;
        Self(values)
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

impl ByteClass {
    pub(crate) fn iter(&self) -> Iter<u8> {
        self.0.iter()
    }
}
