use super::*;

use std::collections::{BTreeMap, BTreeSet, LinkedList};

#[derive(Debug, Clone)]
pub struct NFA {
    // Reduce to a single init state, namely the root
    pub(crate) start: StateId,
    pub(crate) states: BTreeMap<StateId, State>,
    pub(crate) end: StateId,
}

#[derive(Debug, Clone)]
pub(crate) struct State {
    table: BTreeMap<Option<char>, LinkedList<StateId>>,
}

impl State {
    fn empty() -> Self {
        Self {
            table: Default::default(),
        }
    }

    pub(crate) fn get(&self, k: &Option<char>) -> Option<&LinkedList<StateId>> {
        self.table.get(k)
    }

    fn insert(&mut self, k: Option<char>, v: StateId) {
        self.table.entry(k).or_default().push_front(v)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Option<char>, &StateId)> {
        self.table
            .iter()
            .flat_map(|(k, v)| v.iter().map(move |id| (k, id)))
    }

    fn offset(&self, offset: u32) -> Self {
        Self {
            table: self
                .table
                .iter()
                .map(|(k, v)| (*k, v.iter().map(|id| id.add(offset)).collect()))
                .collect(),
        }
    }
}

impl iter::FromIterator<(Option<char>, StateId)> for State {
    fn from_iter<I: IntoIterator<Item = (Option<char>, StateId)>>(iter: I) -> Self {
        let mut state = State::empty();
        for (k, v) in iter.into_iter() {
            state.insert(k, v)
        }
        state
    }
}

impl NFA {
    const EPSILON: Option<char> = None;

    fn empty() -> Self {
        Self {
            start: StateId::of(0),
            states: vec![(StateId::of(0), State::empty())].into_iter().collect(),
            end: StateId::of(0),
        }
    }

    pub(crate) fn get(&self, id: StateId) -> Option<&State> {
        self.states.get(&id)
    }

    pub(crate) fn get_mut(&mut self, id: StateId) -> Option<&mut State> {
        self.states.get_mut(&id)
    }

    fn insert(&mut self, id: StateId, state: State) -> Option<State> {
        self.states.insert(id, state)
    }

    fn push_state(&mut self) -> StateId {
        let id = StateId::of(self.states.len() as u32);
        self.states.insert(id, State::empty());
        id
    }

    pub(crate) fn repeat(mut self) -> Self {
        let new_start = self.push_state();
        let new_end = self.push_state();
        let old_start = self.start;
        let old_end = self.end;

        self.start = new_start;
        self.end = new_end;
        
        {
            let new_start = self.get_mut(new_start).unwrap();
            new_start.insert(Self::EPSILON, new_end);
            new_start.insert(Self::EPSILON, old_start);

            let old_end = self.get_mut(old_end).unwrap();
            old_end.insert(Self::EPSILON, new_end);
        }

        self.get_mut(old_end)
            .unwrap()
            .insert(Self::EPSILON, old_start);

        self
    }

    pub(crate) fn union(mut self, other: &Self) -> Self {
        let new_start = self.push_state();
        let new_end = self.push_state();
        let old_start = self.start;
        let old_end = self.end;

        self.start = new_start;
        self.end = new_end;

        let offset = self.states.len() as u32;
        
        self.states.extend(
            other
                .states
                .iter()
                .map(|(k, v)| (k.add(offset), v.offset(offset))),
        );

        {
            let new_start = self.get_mut(new_start).unwrap();
            new_start.insert(Self::EPSILON, old_start);
            new_start.insert(Self::EPSILON, other.start.add(offset));

            let old_end = self.get_mut(old_end).unwrap();
            old_end.insert(Self::EPSILON, new_end);
            let other_end = self.get_mut(other.end.add(offset)).unwrap();
            other_end.insert(Self::EPSILON, new_end);
        }
        self
    }

    pub(crate) fn not(mut self) -> Self {
        mem::swap(&mut self.start, &mut self.end);
        self
    }

    pub(crate) fn concat(mut self, other: &Self) -> Self {
        let offset = self.states.len() as u32;
        self.states.extend(
            other
                .states
                .iter()
                .map(|(k, v)| (k.add(offset), v.offset(offset))),
        );
        let end = self.get_mut(self.end).unwrap();
        end.insert(Self::EPSILON, other.start.add(offset));
        self.end = other.end.add(offset);
        self
    }

    pub fn find(&self, input: &str) -> bool {
        let mut stack = vec![(self.start, input)];
        while let Some((id, input)) = stack.pop() {
            if let Some(c) = input.chars().next() {
                if let Some(ids) = self.get(id).unwrap().table.get(&Some(c)) {
                    stack.extend(ids.iter().map(|id| (*id, &input[c.len_utf8()..])));
                }
            } else if self.end == id {
                return true;
            }
            if let Some(ids) = self.get(id).unwrap().table.get(&Self::EPSILON) {
                stack.extend(ids.iter().map(|id| (*id, input)));
            }
        }
        false
    }

    pub(crate) fn epsilon_closure(&self, mut states: BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut stack: Vec<_> = states.iter().copied().collect();
        while let Some(q) = stack.pop() {
            for q_e in self
                .get(q)
                .unwrap()
                .get(&Self::EPSILON)
                .iter()
                .flat_map(|ids| ids.iter())
            {
                if states.insert(*q_e) {
                    stack.push(*q_e)
                }
            }
        }
        states
    }

    pub(crate) fn go(&self, states: &BTreeSet<StateId>, c: char) -> BTreeSet<StateId> {
        states
            .iter()
            .copied()
            .flat_map(|id| {
                self.get(id).into_iter().flat_map(|state| {
                    state
                        .get(&Some(c))
                        .into_iter()
                        .flat_map(|ids| ids.iter().copied())
                })
            })
            .collect()
    }
}

impl<'a> From<&Pattern<'a>> for NFA {
    fn from(pattern: &Pattern) -> Self {
        match pattern {
            Pattern::Literal(lit) => {
                let mut nfa = NFA::empty();
                let end = lit.chars().fold(nfa.start, |id, c| {
                    let next = nfa.push_state();
                    nfa.insert(id, iter::once((Some(c), next)).collect());
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
                let mut nfas = one_of.chars().map(|c| {
                    let mut nfa = NFA::empty();
                    let end = nfa.push_state();
                    nfa.end = end;
                    nfa.get_mut(nfa.start).unwrap().insert(Some(c), end);
                    nfa
                });

                if let Some(first) = nfas.next() {
                    nfas.fold(first, |acc, nfa| acc.union(&nfa))
                } else {
                    NFA::empty()
                }
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
}
