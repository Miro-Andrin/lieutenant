use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::Debug,
    iter,
    ops::{Index, IndexMut},
    usize,
};

pub use super::nfa_ops::*;
use super::{
    byteclass::{ByteClass, ByteClassId},
    state::{NfaState, StateId},
    AutomataBuildError,
};
use indexmap::IndexSet;

/*
The nfa representation we are using here is almost like a DFA, with the only addition of having
epsilon/lambda transitions. That way we can convert part of the NFA into a DFA.
*/

#[derive(Debug, Clone)]
pub struct NFA {
    pub(crate) states: Vec<NfaState>,
    pub(crate) translations: IndexSet<ByteClass>,
    pub(crate) ends: HashSet<StateId>,
}

impl Default for NFA {
    fn default() -> Self {
        Self {
            states: Default::default(),
            translations: iter::once(ByteClass::zeros()).collect(),
            ends: Default::default(),
        }
    }
}

impl Index<ByteClassId> for NFA {
    type Output = ByteClass;
    fn index(&self, ByteClassId(index): ByteClassId) -> &Self::Output {
        &self.translations[index as usize]
    }
}

impl Index<StateId> for NFA {
    type Output = NfaState;
    fn index(&self, StateId(index): StateId) -> &Self::Output {
        &self.states[index as usize]
    }
}

impl IndexMut<StateId> for NFA {
    fn index_mut(&mut self, StateId(index): StateId) -> &mut Self::Output {
        &mut self.states[index as usize]
    }
}

impl NFA {
    pub fn empty() -> Self {
        Default::default()
    }

    /**
        Returns the state acsessible from StateId with a transition/edge that consumes the u8.
        Does not follow any epsilons transitions.
    */
    pub(crate) fn edge(&self, from: StateId, edge: u8) -> Option<StateId> {
        let state = &self[from];
        let index = self[state.class.clone()][edge];
        if index == 0 {
            None
        } else {
            Some(state[index - 1])
        }
    }

    pub(crate) fn with_capacity(states: usize, byteclasses: usize, ends: usize) -> Self {
        let mut res = Self {
            states: Vec::with_capacity(states),
            translations: IndexSet::with_capacity(byteclasses),
            ends: HashSet::with_capacity(ends),
        };

        res.translations.insert(ByteClass::zeros());
        res
    }
    /**
        Adds end: StateId as a end state and returns a bool telling if it already was
        a end state.
    */
    pub(crate) fn push_end(&mut self, end: StateId) -> bool {
        if self.ends.contains(&end) {
            false
        } else {
            self.ends.insert(end);
            true
        }
    }

    /*
        Crates a new state and returns its id.
    */
    pub(crate) fn push_state(&mut self) -> StateId {
        self.states.push(NfaState::default());
        StateId::of(self.states.len() - 1)
    }

    /**
        Returns if this NFA does not contain any states. Note that this is different
        then it not accepting any input.
    */
    pub(crate) fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /**
        Returns if the given state is a end state.
    */
    pub(crate) fn is_end(&self, state: &StateId) -> bool {
        self.ends.contains(state)
    }

    /**
        Adds a epsilon transition, aka a lambda transition.
    */
    pub(crate) fn push_epsilon(&mut self, from: StateId, to: StateId) {
        self[from].push_epsilon(to);
    }

    /**
        Creates a connection between the two states. This potentially
        creates a new byteclass that encodes the transition.
    */
    pub(crate) fn push_connections<Itr: IntoIterator<Item = u8>>(
        &mut self,
        from: StateId,
        to: StateId,
        values: Itr,
    ) -> Result<(), AutomataBuildError> {
        let existing_byteclass = &self[self[from].class];
        let mut new_byteclass = existing_byteclass.clone();
        let mut unhandeld = Vec::new();

        for c in values {
            match new_byteclass[c] {
                0 => {
                    // This means that self has no existing connection for input c
                    // from 'from'.

                    let index = match self[from].table.iter().position(|x| *x == to) {
                        Some(index) => index + 1,
                        None => {
                            // Add 'to' as neighbour of 'from'
                            self[from].table.push(to);
                            self[from].table.len()
                        }
                    };

                    // TODO: this conversion might fail in theory
                    new_byteclass[c] = index as u8;
                }
                _ => {
                    // This means that there already exists a connection from 'from' to 'to'
                    unhandeld.push(c);
                }
            }
        }

        if !unhandeld.is_empty() {
            let stopgap = self.push_state();
            self.push_epsilon(from, stopgap);
            self.push_connections(stopgap, to, unhandeld)?;
        }
        // TODO: This leaves potentially the existing byteclass as unused therefor
        // this can lead to buildup of garbage that needs to be handeld.
        let (class_index, _) = self.translations.insert_full(new_byteclass);
        self[from].class = ByteClassId::try_from(class_index)?;
        Ok(())
    }
    /**
        Could create a new byteclass per call, so use push_connections should be used if you plan on
        adding a lot of connections between two states.
    */
    pub(crate) fn push_connection(
        &mut self,
        from: StateId,
        to: StateId,
        byte: u8,
    ) -> Result<(), AutomataBuildError> {
        let mut state = std::mem::take(&mut self[from]);

        match self[state.class.clone()][byte] {
            0 => {
                // This means that self has no existing connection for 'c'.
                let mut new_byteclass = self[state.class].clone();

                state.table.push(to);
                let index = state.table.len();

                // Update class
                new_byteclass[byte] = index as u8;
                let (class_index, _) = self.translations.insert_full(new_byteclass);
                state.class = ByteClassId::try_from(class_index)?;

                // TODO: We could now run a GC like procedure on the nfa, because nfa.transitions might contain a unused byteclass.
                // this might be reasonable to do.

                self[from] = state;
            }
            _ => {
                // This means that there already exists index
                self[from] = state;
                let neighbour = self.push_state();
                self.push_epsilon(from, neighbour);
                self.push_connection(neighbour, to, byte)?;
            }
        };

        Ok(())
    }

    /**
        Assosiates the nfa with the id, that is a index into the command vec in the
        dispatcher. This keeps track of that command the given nfa belongs to.
    */
    pub fn assosiate_with(&mut self, id: usize) {
        // Assosiate the nfa with the id,
        for state in self.states.iter_mut() {
            state.assosiations.insert(id);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_push_connections_a() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        nfa.push_connections(start, end, 97..=97).unwrap();
        let a = nfa.find("a");
        assert!(a.is_ok());
    }

    #[quickcheck]
    fn test_push_connections(from: u8, to: u8, other: u8) -> bool {
        if from > to {
            return true;
        }

        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        nfa.push_connections(start, end, from..=to).unwrap();
        (from <= other && other <= to) == nfa.find(&[other]).is_ok()
    }

    #[test]
    fn test_push_connections3() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        let a = nfa.push_state();
        let b = nfa.push_state();
        nfa.push_end(end);

        nfa.push_connections(start, a, 0..=100).unwrap();
        nfa.push_connections(start, b, 50..=150).unwrap();
        nfa.push_connections(a, end, 0..50).unwrap();
        nfa.push_connections(b, end, 50..=100).unwrap();

        assert!(nfa.find(&[50, 25]).is_ok());
        assert!(nfa.find(&[50, 75]).is_ok());

        println!("{:?}", nfa);
    }

    #[test]
    fn test_push_connections_b() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);

        nfa.push_connections(start, end, 10..=150).unwrap();
        let state = &nfa[start];
        assert!(state.table.len() < 2);
        let a = nfa.find("a");
        assert!(a.is_ok());
    }

    #[test]
    fn test_push_connection_a() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);

        nfa.push_connection(start, end, 97).unwrap();

        let a = nfa.find("a");
        assert!(a.is_ok());
    }

    #[test]
    fn test_connection() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end1 = nfa.push_state();
        let end2 = nfa.push_state();

        nfa.push_end(end1);
        nfa.push_end(end2);

        nfa.push_connection(start, end1, 5).unwrap();
        nfa.push_connection(start, end2, 5).unwrap();

        assert!(nfa.find(&[5]).is_ok());
        println!("{:?}", nfa);
        assert!(nfa.find(&[5]).unwrap().len() == 2);
    }

    #[test]
    fn test_epsilon() {
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_epsilon(start, end);
        println!("{}", nfa);
    }
}
