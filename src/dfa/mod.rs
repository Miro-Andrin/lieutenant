use crate::command::Command;
use crate::error::Result;
use crate::graph::{NodeId, NodeKind, ParserKind, RootNode};
use std::collections::HashMap;
use std::iter;
use std::ops::{Index, IndexMut};

use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StateId(u32);

impl StateId {
    fn of(id: u32) -> Option<Self> {
        if id == u32::MAX {
            None
        } else {
            Some(Self(id))
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeKindId(u32);

impl NodeKindId {
    fn of(id: u32) -> Option<Self> {
        if id == u32::MAX {
            None
        } else {
            Some(Self(id))
        }
    }
}

#[derive(Debug)]
struct DFA {
    states: Vec<State>,
    node_kinds: Vec<NodeKind>,
    end_states: HashMap<StateId, usize>,
}

impl Index<StateId> for DFA {
    type Output = State;
    fn index(&self, index: StateId) -> &Self::Output {
        &self.states[index.0 as usize]
    }
}

impl IndexMut<StateId> for DFA {
    fn index_mut(&mut self, index: StateId) -> &mut Self::Output {
        &mut self.states[index.0 as usize]
    }
}

#[derive(Debug)]
pub struct State {
    table: Vec<Option<StateId>>,
    node_kinds: Vec<NodeKindId>,
    stride: u8,
}

impl State {
    fn empty() -> State {
        State {
            table: vec![],
            node_kinds: vec![],
            stride: 0,
        }
    }

    fn get(&self, byte: u8) -> Option<StateId> {
        if let Some(index) = byte.checked_sub(self.stride) {
            self.table.get(index as usize).copied().flatten()
        } else {
            None
        }
    }

    fn set(&mut self, byte: u8, state_id: Option<StateId>) -> Option<StateId> {
        let prev = self.get(byte);
        if state_id.is_none() {
            if self.table.is_empty() {
            } else if byte > self.stride && byte < self.table.len() as u8 {
                self.table[(byte - self.stride) as usize] = state_id;
            }
        } else if self.table.is_empty() {
            self.stride = byte;
            self.table.push(state_id);
        } else if byte < self.stride {
            // Extend front
            let mut table = Vec::with_capacity(self.table.len() + (self.stride - byte) as usize);
            table.push(state_id);
            table.extend(
                iter::repeat(None)
                    .take((self.stride - byte) as usize)
                    .skip(1),
            );
            table.extend(self.table.drain(..));
            self.table = table;
            self.stride = byte;
        } else if ((byte - self.stride) as usize) < self.table.len() {
            // Middle
            self.table[(byte - self.stride) as usize] = state_id;
        } else {
            // Extend back
            self.table
                .extend(iter::repeat(None).take((byte - self.stride) as usize - self.table.len()));
            self.table.push(state_id);
        }
        prev
    }
}

impl DFA {
    fn get(&self, state_id: StateId) -> Option<&State> {
        self.states.get(state_id.0 as usize)
    }

    fn get_mut(&mut self, state_id: StateId) -> Option<&mut State> {
        self.states.get_mut(state_id.0 as usize)
    }

    fn start(&self) -> Option<&State> {
        self.get(StateId::of(0).unwrap())
    }

    fn end_state(&self, state_id: StateId) -> Option<usize> {
        self.end_states.get(&state_id).copied()
    }

    fn find(&self, input: &str) -> Option<usize> {
        let mut bytes = input.bytes();
        let mut state_id = StateId::of(0);
        while let Some(byte) = bytes.next() {
            if let Some(state) = state_id.and_then(|id| self.get(id)) {
                state_id = state.get(byte);
            } else {
                return None;
            }
        }
        state_id.and_then(|id| self.end_state(id))
    }

    fn push_state(&mut self) -> StateId {
        let id = StateId(self.states.len() as u32);
        self.states.push(State::empty());
        id
    }
}

#[derive(Clone, Debug)]
struct StateIds(Vec<StateId>);

impl std::iter::FromIterator<StateId> for StateIds {
    fn from_iter<T: IntoIterator<Item = StateId>>(iter: T) -> Self {
        StateIds(iter.into_iter().collect())
    }
}

impl std::iter::FromIterator<Option<StateId>> for StateIds {
    fn from_iter<T: IntoIterator<Item = Option<StateId>>>(iter: T) -> Self {
        StateIds(iter.into_iter().filter_map(|id| id).collect())
    }
}

impl std::iter::IntoIterator for StateIds {
    type Item = StateId;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl StateIds {
    fn iter(&self) -> impl Iterator<Item = &StateId> {
        self.0.iter()
    }

    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = StateId>,
    {
        self.0.extend(iter)
    }

    fn last(&self) -> Option<&StateId> {
        self.0.last()
    }

    fn first(&self) -> Option<&StateId> {
        self.0.first()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> Option<&StateId> {
        self.get(index)
    }
}

impl StateId {
    const ROOT: StateId = StateId(0);

    fn transition(&self, dfa: &mut DFA, byte: u8, to: Option<StateId>) -> StateId {
        let to = to.unwrap_or_else(|| dfa.push_state());
        dfa[self.clone()].set(byte, Some(to));
        to
    }

    fn one_of(&self, bytes: &[u8], dfa: &mut DFA, to: Option<StateId>) -> Option<StateId> {
        bytes.iter().copied().fold(to, |to, byte| Some(self.transition(dfa, byte, to)))
    }

    fn literal(&self, bytes: &[u8], dfa: &mut DFA, to: Option<StateId>) -> Option<StateId> {
        let mut state = self.clone();
        for byte in &bytes[0..bytes.len() - 1] {
            state = state.one_of(&[*byte], dfa, None).unwrap();
        }
        if let Some(last) = bytes.last() {
            state.one_of(&[*last], dfa, to)
        } else {
            None
        }
    }
}

impl StateIds {
    fn transition(&self, dfa: &mut DFA, byte: u8, to: Option<StateId>) {
        for state_id in self.iter().copied() {
            dfa[state_id].set(byte, to);
        }
    }
}

impl DFA {
    fn new() -> Self {
        Self {
            states: vec![State::empty()],
            node_kinds: vec![],
            end_states: Default::default(),
        }
    }

    fn build<Ctx>(
        mut root: RootNode<Ctx>,
    ) -> (Self, Vec<Box<dyn Command<Ctx = Ctx, Output = Result<()>>>>) {
        let mut dfa = DFA::new();
        let mut stack = root
            .children
            .drain(..)
            .map(|id| (StateId::ROOT, id))
            .collect::<Vec<_>>();

        let mut execs = vec![];
        while let Some((mut state, node_id)) = stack.pop() {
            let mut node = root.remove(node_id);

            let node_kind_id = NodeKindId(dfa.node_kinds.len() as u32);

            state = match &node.kind {
                NodeKind::Literal(lit) => match state.literal(lit.as_bytes(), &mut dfa, None) {
                    None => continue,
                    Some(state) => state,
                },
                NodeKind::Argument { parser } => match parser {
ParserKind::IntRange => {
    let state = state.one_of("0123456789".as_bytes(), &mut dfa, None).unwrap();
    state.one_of("0123456789".as_bytes(), &mut dfa, Some(state));
    state
}
                    _ => todo!(),
                },
            };

            dfa.node_kinds.push(node.kind);

            if let Some(exec) = node.execute.take() {
                dfa.end_states.insert(state, execs.len());
                execs.push(exec);
            }

            state = state.one_of(" ".as_bytes(), &mut dfa, None).unwrap();
            state.one_of(" ".as_bytes(), &mut dfa, Some(state)).unwrap();
            
            stack.extend(node.children.drain(..).map(|id| (state.clone(), id)));
        }
        (dfa, execs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{BlankBuilder, CommandBuilder};
    use crate::graph::RootNode;
    use crate::values::Values;

    #[test]
    fn test() {
        let root: RootNode<()> = BlankBuilder::new().literal("tp").build(|| {
            println!("it works!");
            Ok(())
        });
        let (dfa, execs) = DFA::build(root);

        assert_eq!(
            dfa[StateId::of(0).unwrap()].get('t' as u8),
            Some(StateId(1))
        );
        assert_eq!(
            dfa[StateId::of(1).unwrap()].get('p' as u8),
            Some(StateId(2))
        );
        assert!(dfa.end_state(StateId(2)).is_some());

        dbg!(&dfa);

        let command = dfa.find("tp");
        dbg!(&command.is_some());
        if let Some(exec) = command.and_then(|i| execs.get(i)) {
            exec.invoke(&mut (), &mut Values::from(&vec![]));
        }
    }

    #[test]
    fn test_int() {
        let root: RootNode<()> = BlankBuilder::new().literal("tp").param().build(|_: i32| {
            println!("it works!");
            Ok(())
        });
        let (dfa, execs) = DFA::build(root);

        let command = dfa.find("tp 1222222");
        if let Some(exec) = command.and_then(|index| execs.get(index)) {
            exec.invoke(&mut (), &mut Values::from(&vec![]));
        }
    }
}
