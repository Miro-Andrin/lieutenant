use super::*;

use indexmap::IndexSet;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct NFA {
    // Reduce to a single init state, namely the root
    pub(crate) start: StateId,
    pub(crate) states: Vec<State>,
    pub(crate) end: StateId,
    pub(crate) classes: IndexSet<ByteClass>,
}

impl Index<ByteClassId> for NFA {
    type Output = ByteClass;
    fn index(&self, ByteClassId(index): ByteClassId) -> &Self::Output {
        &self.classes[index as usize]
    }
}

impl Index<StateId> for NFA {
    type Output = State;
    fn index(&self, StateId(index): StateId) -> &Self::Output {
        &self.states[index as usize]
    }
}

impl IndexMut<StateId> for NFA {
    fn index_mut(&mut self, StateId(index): StateId) -> &mut Self::Output {
        &mut self.states[index as usize]
    }
}

impl Index<(StateId, Option<u8>)> for NFA {
    type Output = Vec<StateId>;
    fn index(&self, (id, step): (StateId, Option<u8>)) -> &Self::Output {
        let state = &self[id];
        match step {
            Some(b) => &state[self[state.class][b]],
            None => &state.epsilons,
        }
    }
}

impl Index<(StateId, u8)> for NFA {
    type Output = Vec<StateId>;
    fn index(&self, (id, b): (StateId, u8)) -> &Self::Output {
        let state = &self[id];
        &state[self[state.class][b]]
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub(crate) table: Vec<Vec<StateId>>,
    pub(crate) class: ByteClassId,
    pub(crate) epsilons: Vec<StateId>,
}

impl Index<u8> for State {
    type Output = Vec<StateId>;
    fn index(&self, index: u8) -> &Self::Output {
        &self.table[index as usize]
    }
}

impl IndexMut<u8> for State {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.table[index as usize]
    }
}

impl State {
    fn empty() -> Self {
        Self {
            table: vec![vec![]],
            class: ByteClassId(0),
            epsilons: vec![],
        }
    }

    fn push_epsilon(&mut self, to: StateId) {
        self.epsilons.push(to);
    }
}

impl NFA {
    fn empty() -> Self {
        Self {
            start: StateId::of(0),
            states: vec![State::empty()],
            classes: iter::once(ByteClass::empty()).collect(),
            end: StateId::of(0),
        }
    }

    fn push_state(&mut self) -> StateId {
        let id = StateId::of(self.states.len() as u32);
        self.states.push(State::empty());
        id
    }

    fn extend(&mut self, other: &Self) -> (StateId, StateId) {
        let offset = self.states.len() as u32;
        let mut states = mem::take(&mut self.states);
        states.extend(other.states.iter().map(|state| {
            let mut state = state.clone();
            state
                .table
                .iter_mut()
                .for_each(|ids| ids.iter_mut().for_each(|id| *id = id.add(offset)));

            let byte_class = &other[state.class];
            let class_id = self.push_class(byte_class.clone());
            state.class = class_id;
            state
        }));

        self.states = states;

        (other.start.add(offset), other.end.add(offset))
    }

    pub(crate) fn repeat(mut self) -> Self {
        let new_start = self.push_state();
        let new_end = self.push_state();
        let old_start = self.start;
        let old_end = self.end;

        self.start = new_start;
        self.end = new_end;

        let new_start = &mut self[new_start];
        new_start.push_epsilon(new_end);
        new_start.push_epsilon(old_start);

        let old_end = &mut self[old_end];
        old_end.push_epsilon(new_end);
        old_end.push_epsilon(old_start);

        self
    }

    pub(crate) fn union(mut self, other: &Self) -> Self {
        let new_start = self.push_state();
        let new_end = self.push_state();
        let old_start = self.start;
        let old_end = self.end;

        self.start = new_start;
        self.end = new_end;

        let (other_start, other_end) = self.extend(other);

        let new_start = &mut self[new_start];
        new_start.push_epsilon(old_start);
        new_start.push_epsilon(other_start);

        let old_end = &mut self[old_end];
        old_end.push_epsilon(new_end);

        let other_end = &mut self[other_end];
        other_end.push_epsilon(new_end);

        self
    }

    pub(crate) fn not(mut self) -> Self {
        mem::swap(&mut self.start, &mut self.end);
        self
    }

    pub(crate) fn concat(mut self, other: &Self) -> Self {
        let (other_start, other_end) = self.extend(other);
        let end = self.end;
        let end = &mut self[end];
        end.push_epsilon(other_start);
        self.end = other_end;
        self
    }

    pub fn find(&self, input: &str) -> bool {
        let mut stack = vec![(self.start, input.as_bytes())];
        while let Some((id, input)) = stack.pop() {
            if let Some(b) = input.get(0) {
                stack.extend(self[(id, *b)].iter().map(|id| (*id, &input[1..])));
            } else if self.end == id {
                return true;
            }
            stack.extend(self[id].epsilons.iter().map(|id| (*id, input)));
        }
        false
    }

    pub(crate) fn epsilon_closure(&self, mut states: BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut stack: Vec<_> = states.iter().copied().collect();
        while let Some(q) = stack.pop() {
            for q_e in self[q].epsilons.iter() {
                if states.insert(*q_e) {
                    stack.push(*q_e)
                }
            }
        }
        states
    }

    pub(crate) fn go(&self, states: &BTreeSet<StateId>, b: u8) -> BTreeSet<StateId> {
        states
            .iter()
            .copied()
            .flat_map(|id| self[(id, b)].iter().copied())
            .collect()
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

    pub(crate) fn set_transitions<I, A>(
        &mut self,
        id: StateId,
        byte_class: ByteClass,
        transitions: I,
    ) where
        I: IntoIterator<Item = A>,
        A: IntoIterator<Item = StateId>,
    {
        let class_id = self.push_class(byte_class);
        self[id].class = class_id;
        self[id].table = transitions
            .into_iter()
            .map(|ids| ids.into_iter().collect())
            .collect();
    }
}

impl<'a> From<&Pattern<'a>> for NFA {
    fn from(pattern: &Pattern) -> Self {
        match pattern {
            Pattern::Literal(lit) => {
                let mut nfa = NFA::empty();
                let end = lit.bytes().fold(nfa.start, |id, c| {
                    let next = nfa.push_state();

                    let byte_class = ByteClass::from(c..c);
                    nfa.set_transitions(id, byte_class, vec![vec![], vec![next], vec![]]);

                    next
                });
                nfa.end = end;
                nfa
            }
            Pattern::Many(pattern) => NFA::from(*pattern).repeat(),
            Pattern::Concat(patterns) => patterns
                .iter()
                .fold(NFA::from(&pattern!("")), |nfa, pattern| {
                    nfa.concat(&NFA::from(pattern))
                }),
            Pattern::Alt(patterns) => patterns
                .iter()
                .fold(NFA::from(&pattern!("")), |nfa, pattern| {
                    nfa.union(&NFA::from(pattern))
                }),
            Pattern::OneOf(one_of) => {
                let mut buffer = [0; 4];
                let mut classes = vec![ByteClass::empty(); 4];
                for c in one_of.chars() {
                    let bytes = c.encode_utf8(&mut buffer);
                    for (i, b) in bytes.bytes().enumerate() {
                        if i + 1 < c.len_utf8() {
                            classes[i][b] = 2;
                        } else {
                            classes[i][b] = 1;
                        }
                    }
                }
                let mut nfa = NFA::empty();
                let mut id = nfa.start;

                let classes: Vec<_> = classes
                    .into_iter()
                    .take_while(|class| !class.is_empty())
                    .collect();

                let end = StateId::of(classes.len() as u32);

                for class in classes {
                    let next_id = nfa.push_state();
                    if next_id == end {
                        nfa.set_transitions(id, class, vec![vec![], vec![end], vec![]])
                    } else {
                        nfa.set_transitions(id, class, vec![vec![], vec![end], vec![next_id]]);
                    }
                    id = next_id;
                }

                nfa.end = id;

                nfa
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc() {
        let nfa_abc = NFA::from(&pattern!("abc"));

        println!("{:?}", nfa_abc);

        assert!(!nfa_abc.find(""));
        assert!(nfa_abc.find("abc"));
        assert!(!nfa_abc.find("abcabc"));
        assert!(!nfa_abc.find("a"));
    }

    #[test]
    fn abc_abc() {
        let nfa_abc_abc = NFA::from(&pattern!("abc" "abc"));

        assert!(!nfa_abc_abc.find(""));
        assert!(!nfa_abc_abc.find("a"));
        assert!(!nfa_abc_abc.find("abc"));
        assert!(nfa_abc_abc.find("abcabc"));
        assert!(!nfa_abc_abc.find("abcabcabc"));
    }

    #[test]
    fn abc_repeat() {
        let nfa_abc_repeat = NFA::from(&pattern!("abc"*));

        assert!(nfa_abc_repeat.find(""));
        assert!(nfa_abc_repeat.find("abc"));
        assert!(nfa_abc_repeat.find("abcabc"));
        assert!(!nfa_abc_repeat.find("a"));
        assert!(!nfa_abc_repeat.find("b"));
    }

    #[test]
    fn abc_abc_repeat() {
        let nfa_abc = NFA::from(&pattern!("abc"));

        let nfa_abc_abc = nfa_abc.clone().concat(&nfa_abc);
        let nfa_abc_abc_repeat = nfa_abc_abc.repeat();

        assert!(nfa_abc_abc_repeat.find(""));
        assert!(!nfa_abc_abc_repeat.find("abc"));
        assert!(nfa_abc_abc_repeat.find("abcabc"));
        assert!(!nfa_abc_abc_repeat.find("abcabcabc"));
    }

    #[test]
    fn abc_u_def() {
        let nfa_abc = NFA::from(&pattern!("abc"));
        let nfa_def = NFA::from(&pattern!("def"));
        let nfa_abc_u_def = nfa_abc.union(&nfa_def);

        assert!(nfa_abc_u_def.find("abc"));
        assert!(nfa_abc_u_def.find("def"));
        assert!(!nfa_abc_u_def.find(""));
        assert!(!nfa_abc_u_def.find("abcd"));
    }

    #[test]
    fn one_of_abc() {
        let nfa_abc = NFA::from(&pattern!(["abc"]));

        assert!(!nfa_abc.find(""));
        assert!(nfa_abc.find("a"));
        assert!(nfa_abc.find("b"));
        assert!(nfa_abc.find("c"));
        assert!(!nfa_abc.find("aa"));
        assert!(!nfa_abc.find("ab"));
        assert!(!nfa_abc.find("ac"));
    }

    #[test]
    fn unicode() {
        let nfa = NFA::from(&pattern!(["æøå🌏"]));

        assert!(nfa.find("æ"));
        assert!(nfa.find("ø"));
        assert!(nfa.find("å"));
        assert!(nfa.find("🌏"));
        assert!(!nfa.find("a"));
        assert!(!nfa.find("b"));
        assert!(!nfa.find("c"));
    }

    #[test]
    fn unicode_repeat() {
        let nfa = NFA::from(&pattern!(["æøå"]*));

        assert!(nfa.find("æå"));
        assert!(nfa.find("øæ"));
        assert!(nfa.find("åø"));
        assert!(!nfa.find("ab"));
        assert!(!nfa.find("bc"));
        assert!(!nfa.find("cd"));
    }
}
