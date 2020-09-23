use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct DFA {
    /// All posible states and their transition table
    pub(crate) states: Vec<State>,
    pub(crate) ends: BTreeSet<StateId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct State {
    pub(crate) table: BTreeMap<char, StateId>,
}

impl Default for State {
    fn default() -> Self {
        Self::empty()
    }
}

impl State {
    fn empty() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }


    fn get(&self, c: char) -> Option<StateId> {
        self.table.get(&c).copied()
    }
}

impl iter::FromIterator<(char, StateId)> for State {
    fn from_iter<I: IntoIterator<Item = (char, StateId)>>(iter: I) -> Self {
        Self {
            table: iter.into_iter().collect(),
        }
    }
}

impl DFA {
    fn get(&self, StateId(index): StateId) -> Option<&State> {
        self.states.get(index as usize)
    }

    fn get_mut(&mut self, StateId(index): StateId) -> Option<&mut State> {
        self.states.get_mut(index as usize)
    }

    pub fn find(&self, input: &str) -> Option<StateId> {
        let mut current = StateId::of(0);
        for c in input.chars() {
            if let Some(next) = self.get(current).unwrap().get(c) {
                current = next;
            } else {
                return None;
            }
        }
        if self.ends.contains(&current) {
            Some(current)
        } else {
            None
        }
    }

    pub fn minimize(mut self) -> Self {
        let mut mapping: BTreeMap<StateId, StateId> = BTreeMap::new();
        let mut new_states: BTreeMap<&State, StateId> = BTreeMap::new();
        let mut new_end_sates: BTreeMap<&State, StateId> = BTreeMap::new();
        let mut next_state_id = 0;

        for (old_id, is_end, state) in self
            .states
            .iter()
            .into_iter()
            .enumerate()
            .map(|(i, state)| (i as u32, self.ends.contains(&StateId::of(i as u32)), state))
        {
            let new_states = if is_end {
                &mut new_end_sates
            } else {
                &mut new_states
            };
            if let Some(new_id) = new_states.get(&state) {
                mapping.insert(StateId::of(old_id), *new_id);
            } else {
                let new_state_id = StateId::of(next_state_id);
                next_state_id += 1;
                new_states.insert(state, new_state_id);
                mapping.insert(StateId::of(old_id), new_state_id);
            }
        }

        let ends = new_end_sates.into_iter().map(|(_, id)| id).collect();

        let mut new_states = BTreeMap::new();
        for (old_id, new_id) in &mapping {
            if !new_states.contains_key(&new_id) {
                let mut state = mem::take(self.get_mut(*old_id).unwrap());
                state
                    .table
                    .iter_mut()
                    .for_each(|(_, id)| *id = *mapping.get(id).unwrap());
                new_states.insert(new_id, state);
            }
        }

        DFA {
            states: new_states.into_iter().map(|(_, state)| state).collect(),
            ends,
        }
    }
}
