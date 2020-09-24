// http://www.cs.nuim.ie/~jpower/Courses/Previous/parsing/node9.html

use super::*;
use std::collections::{BTreeMap, BTreeSet};

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> Self {
        let mut nfa_to_dfa: BTreeMap<BTreeSet<StateId>, StateId> = BTreeMap::new();

        let mut dfa = DFA::empty();

        // Create the start state of the DFA by taking the epsilon_closure of the start state of the NFA.
        let mut stack: Vec<BTreeSet<StateId>> =
            vec![nfa.epsilon_closure(iter::once(nfa.start).collect())];

        while let Some(nfa_ids) = stack.pop() {
            let dfa_id = dfa.push_state();
            nfa_to_dfa.insert(nfa_ids.clone(), dfa_id);

            let mut transitions = vec![];

            // For each possible input symbol
            for b in 0..=255 as u8 {
                // Apply move to the newly-created state and the input symbol; this will return a set of states.
                let move_states = nfa.go(&nfa_ids, b);

                if move_states.is_empty() {
                    transitions.push(None);
                    continue;
                }

                // Apply the epsilon_closure to this set of states, possibly resulting in a new set.
                let move_state_e = nfa.epsilon_closure(move_states);

                let dfa_e_id = if let Some(dfa_e_id) = nfa_to_dfa.get(&move_state_e) {
                    *dfa_e_id
                } else {
                    let dfa_e_id = StateId::of(dfa.states.len() as u32);
                    nfa_to_dfa.insert(move_state_e.clone(), dfa_e_id);

                    // Each time we generate a new DFA state, we must apply step 2 to it. The process is complete when applying step 2 does not yield any new states.
                    stack.push(move_state_e);
                    dfa_e_id
                };
                
                transitions.push(Some(dfa_e_id));
            }

            dfa.set_transitions(dfa_id, transitions);
            if nfa_ids.iter().any(|id| *id == nfa.end) {
                dfa.ends.push(dfa_id);
            }
        }
        dfa
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc_repeat() {
        let nfa_abc_repeat = NFA::from(&pattern!("abc"*));

        let dfa_abc_repeat = DFA::from(nfa_abc_repeat);
    }

    #[test]
    fn abc_u_abc() {
        let nfa_abc = NFA::from(&pattern!("abc"));
        let nfa_def = NFA::from(&pattern!("def"));
        let nfa_abc_u_abc = nfa_abc.union(&nfa_def);

        let nfa = NFA::from(&pattern!("a")).concat(&nfa_abc_u_abc);

        println!("{:?}", nfa);

        let dfa_abc_u_abc = DFA::from(nfa);

        println!("{:?}", dfa_abc_u_abc);
    }
}
