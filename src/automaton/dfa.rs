use super::*;
use std::collections::{BTreeMap};
use indexmap::IndexSet;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct State {
    pub(crate) table: Vec<Option<StateId>>,
    pub(crate) class: ByteClassId,
}

impl Default for State {
    fn default() -> Self {
        Self::empty()
    }
}

impl State {
    fn empty() -> Self {
        Self {
            table: vec![None],
            class: ByteClassId(0),
        }
    }
}

impl Index<u8> for State {
    type Output = Option<StateId>;
    fn index(&self, index: u8) -> &Self::Output {
        &self.table[index as usize]
    }
}

#[derive(Debug, Clone)]
pub struct DFA {
    /// All posible states and their transition table
    pub states: Vec<State>,
    pub(crate) ends: Vec<StateId>,
    pub(crate) classes: IndexSet<ByteClass>,
}

impl Index<StateId> for DFA {
    type Output = State;
    fn index(&self, StateId(index): StateId) -> &Self::Output {
        &self.states[index as usize]
    }
}

impl IndexMut<StateId> for DFA {
    fn index_mut(&mut self, StateId(index): StateId) -> &mut Self::Output {
        &mut self.states[index as usize]
    }
}

impl Index<ByteClassId> for DFA {
    type Output = ByteClass;
    fn index(&self, ByteClassId(index): ByteClassId) -> &Self::Output {
        &self.classes[index as usize]
    }
}

impl Index<(StateId, u8)> for DFA {
    type Output = Option<StateId>;
    fn index(&self, (id, b): (StateId, u8)) -> &Self::Output {
        let state = &self[id];
        &state[self[state.class][b]]
    }
}


impl DFA {
    /// Creates an DFA which matches nothing.
    pub(crate) fn nothing() -> Self {
        Self {
            states: vec![],
            classes: IndexSet::new(),
            ends: vec![],
        }
    }

    /// Creates an DFA which matches the empty string.
    pub fn empty() -> Self {
        Self {
            states: vec![State::empty()],
            classes: iter::once(ByteClass::empty()).collect(),
            ends: vec![StateId::of(0)],
        }
    }

    /// Create a new empty state and returns its id. 
    pub(crate) fn push_state(&mut self) -> StateId {
        let id = StateId::of(self.states.len() as u32);
        self.states.push(State::empty());
        id
    }

    pub(crate) fn set_transitions<I>(&mut self, id: StateId, transitions: I)
    where
        I: IntoIterator<Item = Option<StateId>>,
    {
        let mut table = vec![];
        let mut seen = IndexSet::new();
        let mut class = ByteClass::empty();
        for (b, id) in transitions.into_iter().enumerate() {
            if let Some(i) = seen.get_index_of(&id) {
                class.0[b] = i as u8;
            } else {
                class.0[b] = seen.len() as u8;
                seen.insert(id);
                table.push(id);
            }
        }

        self[id].table = table;
        
        let class_id = self.push_class(class);
        self[id].class = class_id;
    }

    pub(crate) fn push_class(&mut self, class: ByteClass) -> ByteClassId {
        if let Some(id) = self.classes.get_index_of(&class) {
            ByteClassId(id as u16)
        } else {
            let id = ByteClassId(self.classes.len() as u16);
            self.classes.insert(class);
            id
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
                let mut state = mem::take(&mut self[*old_id]);
                state
                    .table
                    .iter_mut()
                    .filter_map(|id| id.as_mut())
                    .for_each(|id| *id = mapping[id]);
                new_states.insert(new_id, state);
            }
        }

        DFA {
            states: new_states.into_iter().map(|(_, state)| state).collect(),
            ends,
            classes: self.classes,
        }
    }
}

impl Find<StateId> for DFA {
    fn find(&self, input: &str) -> Result<StateId, StateId> {
        let mut current = StateId::of(0);
        for b in input.bytes() {
            if let Some(next) = self[(current, b)] {
                current = next;
            } else {
                return Err(current);
            }
        }
        if self.ends.contains(&current) {
            Ok(current)
        } else {
            Err(current)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::automaton::{NFA, pattern::*};
    use std::borrow::Cow;

    #[test]
    fn integer_space_integer() {
        let integer_nfa = NFA::from(Pattern::INTEGER);
        let space_nfa = NFA::from(Pattern::SPACE_MANY_ONE);

        let integer_space_integer_nfa = integer_nfa.clone().concat(&space_nfa).concat(&integer_nfa);


        let nfa = integer_space_integer_nfa;
        let dfa = DFA::from(nfa);

        assert!(dfa.find("10 10").is_ok());
        assert!(dfa.find(" 10 10").is_err());
        assert!(dfa.find("10    10").is_ok());
        assert!(dfa.find("a10 10").is_err());
        assert!(dfa.find("10 10 ").is_err());
    }

    #[test]
    fn abc() {
        let nfa = NFA::from(&literal("abc"));
        
        let dfa = DFA::from(nfa);

        assert!(dfa.find("").is_err());
        assert!(dfa.find("abc").is_ok());
        assert!(dfa.find("abcabc").is_err());
        assert!(dfa.find("a").is_err());
    }

    #[test]
    fn abc_abc() {
        let nfa = NFA::from(&concat(&[Cow::Owned(literal("abc")), Cow::Owned(literal("abc"))]));
        let dfa = DFA::from(nfa);

        assert!(dfa.find("").is_err());
        assert!(dfa.find("a").is_err());
        assert!(dfa.find("abc").is_err());
        assert!(dfa.find("abcabc").is_ok());
        assert!(dfa.find("abcabcabc").is_err());
    }
}