use std::{collections::BTreeSet, fmt::Display};

use crate::automata::{state::StateId, NFA};

/*
    What this file generates for nfa:
        https://en.wikipedia.org/wiki/DOT_(graph_description_language)

    How to visualize:
        https://dreampuf.github.io/GraphvizOnline/


    This trait is used during debuging. It gives a pictoral representation of the nfa.
    However it used to define the display of the nfa and not Debug. That is because it lacks some
    information.
*/

impl NFA {
    pub fn to_dot(&self) -> String {
        let mut result = Vec::<String>::with_capacity(self.states.len());

        result.push("digraph NFA {\n".into());

        // Push a definition for every state
        for (id, state) in self.states.iter().enumerate() {
            // a [label="Foo"];

            if self.ends.contains(&StateId::of(id)) {
                // S_id [label="asoc:{} easoc:{}",shape=box]
                let node = format!(
                    "\tS_{} [label=\"S_{} asoc:{:?}\",shape=box];\n",
                    id, id, state.assosiations
                );
                result.push(node);
            } else {
                let node = format!(
                    "\tS_{} [label=\"S_{} asoc:{:?}\"];\n",
                    id, id, state.assosiations
                );
                result.push(node);
            }
        }

        result.push("\n".into());

        for (id, state) in self.states.iter().enumerate() {
            // a [label="Foo"];
            let from = StateId::of(id);

            for epsilon in &state.epsilons {
                let edge = format!(
                    "\tS_{} -> S_{} [label=\"ε\",style=dotted];\n",
                    from.0, epsilon.0
                );
                result.push(edge);
            }

            // state.table could contain values not refferenced in its byteclass

            let used = self[state.class]
                .iter()
                .filter(|x| **x != 0)
                .cloned()
                .collect::<BTreeSet<u8>>();

            for (index, neighbour) in state.table.iter().enumerate() {
                if used.contains(&((index + 1) as u8)) {
                    let edge = format!(
                        "\tS_{} -> S_{} [label=\"{}\"];\n",
                        id, neighbour.0, state.class.0
                    );
                    result.push(edge);
                }
            }
        }

        result.push("}".into());

        let mut string = String::with_capacity(result.iter().map(|x| x.len()).sum());
        for s in result.into_iter() {
            string.push_str(s.as_str());
        }

        string
    }
}

impl Display for NFA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_dot())
    }
}
