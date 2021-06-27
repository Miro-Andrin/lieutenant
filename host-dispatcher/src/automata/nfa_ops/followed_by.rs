use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
};

use crate::automata::{byteclass::ByteClassId, state::StateId, AutomataBuildError, NFA};

impl NFA {
    /// Adds all the states from other into self, and returns the id of the node that used to be
    /// the beginning (StateId::of(0)) and its end_states.
    pub(crate) fn extend(
        &mut self,
        other: Self,
        ofset: usize,
    ) -> Result<(StateId, HashSet<StateId>), AutomataBuildError> {
        self.states.reserve(other.states.len());

        let state_ofset = ofset;
        let other_ends = other
            .ends
            .iter()
            .map(|x| x.clone().add(state_ofset))
            .collect::<HashSet<_>>();
        let other_start = StateId::of(state_ofset);

        for other_index in 0..other.states.len() {
            let mut state = other.states[other_index].clone();

            // When adding the states from 'other' all the StateId's are shifted by 'state_ofset'
            state.table = state
                .table
                .iter_mut()
                .map(|id| id.add(state_ofset))
                .map(|vec| vec.into())
                .collect();

            // When adding the state from 'other' we need to add its byteclass. We can however
            // not just shift the byteclassid by 'state_ofset', because
            // the byteclass might already be present in the 'self' nfa's  byteclass collection.
            let byteclass = other[state.class].clone();
            let (i, _) = self.translations.insert_full(byteclass.clone());
            state.class = i.try_into()?;

            // Update epsilons, by shifting them by state_ofset
            state.epsilons = state
                .epsilons
                .into_iter()
                .map(|e| e.add(state_ofset))
                .collect();

            if state_ofset + other_index < self.states.len() {
                self.states[state_ofset + other_index] = state;
            } else {
                self.states.push(state);
            }
        }
        Ok((other_start, other_ends))
    }

    /// Makes every end state in self have a epsilon transition to the 'other' nfa.
    /// Note how the end states are modified, so this method does not propegate assosiated
    /// values from self to other.
    pub fn followed_by(&mut self, other: NFA) -> Result<(), AutomataBuildError> {
        if other.is_empty() {
            return Ok(());
        } else if self.is_empty() {
            *self = other;
            return Ok(());
        }

        let self_old_ends = self.ends.clone();

        if self.ends.len() == 1 {
            let end = self.ends.iter().cloned().nth(0).unwrap();
            if end.0 as usize == self.states.len() - 1
                && self[end].class == ByteClassId::try_from(0).unwrap()
            {
                // If self has a single end state, then we can forgo adding a epsilon transition.
                // Currently we only do this optimisation if the end state is the last state.
                // Future work is improving this such that it works when the end state has existing outgoing
                // edges, aka byteclassid not equal to zero, and if the end state is not the last state.
                let state_ofset = self.states.len() - 1;
                let (_other_start, other_ends) = self.extend(other, state_ofset)?;
                self.ends = other_ends;
            }
        } else {
            let state_ofset = self.states.len();
            let (other_start, other_ends) = self.extend(other, state_ofset)?;
            for old_end in self_old_ends {
                self.push_epsilon(old_end, other_start);
            }
            self.ends = other_ends;
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::automata::NFA;

    #[test]
    fn followed_by_simple() {
        let mut a = NFA::literal("Aa").unwrap();
        let b = NFA::literal("Bb").unwrap();
        a.followed_by(b).unwrap();

        assert!(a.states.len() == 5);
        assert!(a.find("A").is_err());
        assert!(a.find("C").is_err());
        assert!(a.find("Aa").is_err());
        assert!(a.find("AaB").is_err());
        assert!(a.find("AaBb").is_ok());
    }

    #[test]
    fn followed_by_or() {
        let a = NFA::literal("Aa").unwrap();
        let b = NFA::literal("Bb").unwrap();
        let c = NFA::literal("Cc").unwrap();

        let mut ab = a.union(b).unwrap();

        ab.followed_by(c).unwrap();

        assert!(ab.find("Aa").is_err());
        assert!(ab.find("Bb").is_err());
        assert!(ab.find("AaCc").is_ok());
        assert!(ab.find("BbCc").is_ok());
    }

    #[test]
    fn followed_by_or2() {
        let mut a = NFA::literal("Aa").unwrap();
        let b = NFA::literal("Bb").unwrap();
        let c = NFA::literal("Cc").unwrap();

        a.followed_by(b).unwrap();
        let abc = a.union(c).unwrap();

        assert!(abc.find("Aa").is_err());
        assert!(abc.find("Bb").is_err());
        assert!(abc.find("AaBb").is_ok());
        assert!(abc.find("Cc").is_ok());
    }
}
