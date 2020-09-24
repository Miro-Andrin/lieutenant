// http://www.cs.nuim.ie/~jpower/Courses/Previous/parsing/node9.html

use super::*;
use std::collections::{BTreeMap, BTreeSet};

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> Self {
        let mut nfa_to_dfa: BTreeMap<BTreeSet<StateId>, StateId> = BTreeMap::new();

        let mut next_dfa_id = 0;

        let mut dfa: BTreeMap<StateId, BTreeMap<char, StateId>> = BTreeMap::new();

        // Create the start state of the DFA by taking the epsilon_closure of the start state of the NFA.
        let mut stack: Vec<BTreeSet<StateId>> =
            vec![nfa.epsilon_closure(iter::once(nfa.start).collect())];

        while let Some(nfa_ids) = stack.pop() {
            let dfa_id = StateId::of(next_dfa_id);
            next_dfa_id += 1;
            nfa_to_dfa.insert(nfa_ids.clone(), dfa_id);

            let transitions = nfa_ids.iter().copied().flat_map(|id| {
                nfa.get(id)
                    .unwrap()
                    .iter()
                    .filter_map(|(c, id)| c.map(|c| (c, *id)))
            });

            let mut dfa_transitions: BTreeMap<char, StateId> = BTreeMap::new();

            // For each possible input symbol
            for (t_c, _) in transitions {
                // Apply move to the newly-created state and the input symbol; this will return a set of states.
                let move_state = nfa.go(&nfa_ids, t_c);

                // Apply the epsilon_closure to this set of states, possibly resulting in a new set.
                let move_state_e = nfa.epsilon_closure(move_state);

                // This set of NFA states will be a single state in the DFA.
                let dfa_e_id = if let Some(dfa_e_id) = nfa_to_dfa.get(&move_state_e) {
                    *dfa_e_id
                } else {
                    let dfa_e_id = StateId::of(next_dfa_id);

                    // Each time we generate a new DFA state, we must apply step 2 to it. The process is complete when applying step 2 does not yield any new states.
                    stack.push(move_state_e);
                    dfa_e_id
                };

                dfa_transitions.insert(t_c, dfa_e_id);
            }

            dfa.insert(dfa_id, dfa_transitions);
        }

        let ends = nfa_to_dfa
            .iter()
            .filter(|(ids, _)| ids.iter().any(|id| *id == nfa.end))
            .map(|(_, id)| id)
            .copied()
            .collect();

        DFA {
            states: dfa
                .into_iter()
                .map(|(_, state)| state.into_iter().collect())
                .collect(),
            ends,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc_repeat() {
        let nfa_abc_repeat = NFA::from(&pattern!("abc"*));
        println!("{:?}", &nfa_abc_repeat);

        let dfa_abc_repeat = DFA::from(nfa_abc_repeat);

        println!("{:?}", dfa_abc_repeat);
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
