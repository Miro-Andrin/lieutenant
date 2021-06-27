use std::{collections::BTreeSet, iter};

use crate::automata::{state::StateId, NFA};

/*
This file contains code that does complete matching on input. Calling
nfa.find("some text") returns the set of states one could walk to from
the root node given the input string.
*/

impl NFA {
    /// Goes through all the states in 'from' and adds all the states one can get to by using epsilon
    /// transitions. Aka any transition that does not consume any input.
    pub(crate) fn epsilon_closure(&self, from: BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut stack: Vec<_> = from.into_iter().collect();
        let mut states = BTreeSet::new();
        while let Some(q) = stack.pop() {
            states.insert(q);
            for q_e in self[q].epsilons.iter() {
                if states.insert(*q_e) {
                    stack.push(*q_e)
                }
            }
        }
        states
    }

    pub(crate) fn go(&self, states: &BTreeSet<StateId>, byte: u8) -> BTreeSet<StateId> {
        states
            .iter()
            .cloned()
            .filter(|id| self[*id].table.len() > 0)
            .map(|id| self.edge(id, byte))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn find<T: AsRef<[u8]>>(&self, text: T) -> Result<BTreeSet<StateId>, BTreeSet<StateId>> {
        let mut bytes = text.as_ref();
        let mut current_states: BTreeSet<StateId> = iter::once(StateId::of(0)).collect::<_>();
        loop {
            match self.find_step(bytes, current_states) {
                (Ok(x), false) => {
                    return Ok(x
                        .into_iter()
                        .filter(|x| self.ends.contains(x))
                        .collect::<BTreeSet<_>>())
                }
                (Err(x), false) => return Err(x),
                (Ok(x), true) => {
                    bytes = &bytes[1..];
                    current_states = x;
                }
                (Err(_), true) => unreachable!("unreachable state in nfa find"),
            }
        }
    }

    fn find_step(
        &self,
        bytes: &[u8],
        mut current_states: BTreeSet<StateId>,
    ) -> (Result<BTreeSet<StateId>, BTreeSet<StateId>>, bool) {
        // Follow epsilons all the way.
        current_states = self.epsilon_closure(current_states);

        if bytes.is_empty() {
            if self.ends.iter().any(|x| current_states.contains(x)) {
                return (Ok(current_states), false);
            } else {
                return (Err(current_states), false);
            }
        }

        let new_states = self.go(&current_states, bytes[0]);

        if new_states.is_empty() && !bytes[1..].len() == 0 {
            return (Err(new_states), false);
        }

        (Ok(new_states), true)
    }


    pub fn find_early_termination<T: AsRef<[u8]>>(&self, text: T) -> Result<BTreeSet<StateId>, BTreeSet<StateId>> {

        let mut possible_commands = self[StateId::of(0)].assosiations.clone();

        let mut bytes = text.as_ref();
        let mut current_states: BTreeSet<StateId> = iter::once(StateId::of(0)).collect::<_>();
        loop {
            match self.find_step(bytes, current_states) {
                (Ok(x), false) => {
                    return Ok(x
                        .into_iter()
                        .filter(|x| self.ends.contains(x))
                        .collect::<BTreeSet<_>>())
                }
                (Err(x), false) => return Err(x),
                (Ok(x), true) => {
                    bytes = &bytes[1..];
                    current_states = x;

                    let current_states_assosiated_values = current_states.iter().map(|x| &self[*x].assosiations ).cloned().reduce(|x,y| x.union(&y).collect()).unwrap();
                    possible_commands.intersect_with(&current_states_assosiated_values);

                    if possible_commands.len() == 1 {
                        // Then we do early termination
                        let command_id = possible_commands.iter().nth(0).unwrap();
                        return Ok(current_states.into_iter().filter(|x| self[*x].assosiations.contains(command_id)).collect());
                    }
                    
                }
                (Err(_), true) => unreachable!("unreachable state in nfa find"),
            }
        }
    }

}

#[cfg(test)]
mod test {
    use crate::automata::NFA;

    #[test]
    fn simple_literal_find() {
        let a = NFA::literal("Abc").unwrap();
        assert!(a.find("Abc").is_ok());
        assert!(a.find("Ab").is_err());
        assert!(a.find("A").is_err());
        assert!(a.find("Abcd").is_err());
    }
}
