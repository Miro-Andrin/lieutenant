use std::{collections::BTreeSet, usize};

use crate::automata::{state::StateId, NFA};

const BRANCH_LIMIT: u8 = 10;
const BRANCH_DEPTH: u8 = 4;
const AVG_SEQ_LEN: usize = 3;

#[derive(Debug)]
pub enum TabCandidateError {
    // The first byte must be
    FirstByteNotInRange(u8),

    SecondByteNotInRange(u8),

    ThirdByteNotInRange(u8),

    FourthByteNotInRange(u8),
}

/**
Tab completes should only return Strings. However the nfa works on the
byte aka u8 level. We therefor need to do some bookeeping along the way
such that we return only valid strings. This struct helps us with this.
*/
#[derive(Debug, Clone)]
struct Candidate {
    string: String,
    // StateId for when string was built.
    string_id: StateId,

    // Current stateid
    id: StateId,

    // Bytes we have collected beyond the string, but that does not
    // complete a propper char yet.
    bytes: [u8; 4],
    num_bytes: u8,

    // How many times has this candidate branched to self.string.
    branch_depth: u8,
}

impl Candidate {
    pub(crate) fn new(start: StateId) -> Self {
        Self {
            string: String::new(),
            string_id: start,
            bytes: [0u8; 4],
            num_bytes: 0,
            id: start,
            branch_depth: 0,
        }
    }

    pub(crate) fn push_byte(&mut self, byte: u8, id: StateId) -> Result<(), TabCandidateError> {
        self.bytes[self.num_bytes as usize] = byte;
        self.num_bytes += 1;
        self.id = id;

        match self.bytes[0] {
            0..=191 => {
                // Length is one for the char we are building

                match std::str::from_utf8(&self.bytes[0..1]) {
                    Ok(x) => {
                        self.num_bytes = 0;
                        self.string.push_str(x);
                        self.string_id = id;
                        return Ok(());
                    }
                    Err(_) => {
                        unreachable!(
                            "Earlier check should not have triggerd if the bytes are not utf8"
                        )
                    }
                }
            }
            192..=223 => {
                // Length is two for the char we are building
                if self.num_bytes == 2 {
                    match std::str::from_utf8(&self.bytes[0..2]) {
                        Ok(x) => {
                            self.num_bytes = 0;
                            self.string_id = id;
                            self.string.push_str(x);
                            return Ok(());
                        }
                        Err(_) => {
                            return Err(TabCandidateError::SecondByteNotInRange(self.bytes[1]));
                        }
                    }
                }
                return Ok(());
            }
            224..=239 => {
                // Length is three for the char we are building
                if self.num_bytes == 3 {
                    match std::str::from_utf8(&self.bytes[0..3]) {
                        Ok(x) => {
                            self.num_bytes = 0;
                            self.string_id = id;
                            self.string.push_str(x);
                            return Ok(());
                        }
                        Err(_) => {
                            return Err(TabCandidateError::ThirdByteNotInRange(self.bytes[2]));
                        }
                    }
                }

                if self.num_bytes == 2 {
                    // Check that the second byte is a valid second byte for a 3 long utf-8
                    if !(128..=191).contains(&self.bytes[1]) {
                        return Err(TabCandidateError::SecondByteNotInRange(self.bytes[1]));
                    }
                }

                return Ok(());
            }
            240..=247 => {
                // Length is four for the char we are building
                if self.num_bytes == 4 {
                    match std::str::from_utf8(&self.bytes[0..4]) {
                        Ok(x) => {
                            self.num_bytes = 0;
                            self.id = id;
                            self.string.push_str(x);
                            return Ok(());
                        }
                        Err(_) => {
                            return Err(TabCandidateError::FourthByteNotInRange(self.bytes[3]));
                        }
                    }
                }

                if self.num_bytes == 2 {
                    // Check that the third/second byte is a valid third/second byte for a 4 long utf-8
                    if !(128..=191).contains(&self.bytes[1]) {
                        return Err(TabCandidateError::SecondByteNotInRange(self.bytes[1]));
                    }
                }

                if self.num_bytes == 3 {
                    // Check that the third/second byte is a valid third/second byte for a 4 long utf-8
                    if !(128..=191).contains(&self.bytes[2]) {
                        return Err(TabCandidateError::ThirdByteNotInRange(self.bytes[1]));
                    }
                }

                return Ok(());
            }
            _ => {
                //@TODO remove asert and give better error
                assert!(false, "not a valid start on suposed utf-8 bytes");
                return Err(TabCandidateError::FirstByteNotInRange(self.bytes[0]));
            }
        }
    }
}

impl NFA {
    pub fn tab_complete(&self, from: BTreeSet<StateId>) -> Vec<String> {
        let mut result = Vec::new();

        // We expect the average number of candidates to be something like
        // BRANCH_LIMIT/2 * BRANCH_DEPTH * AVG_SEQ_LEN but the divide by two
        // is just a guess. Average branching is probably just a constant between 1 and 2.
        let mut candidates: Vec<Candidate> = Vec::with_capacity(
            (BRANCH_LIMIT as usize) / 2 * (BRANCH_DEPTH as usize) * (AVG_SEQ_LEN as usize),
        );

        // Setup inital candidates.
        self.epsilon_closure(from)
            .into_iter()
            .for_each(|id| candidates.push(Candidate::new(id)));

        while !candidates.is_empty() {
            let mut candidate = candidates.pop().unwrap();
            let state = &self[candidate.id];
            let byteclass = &self[state.class];

            let branchings: usize = state.table.len();
            let epsilons = state.epsilons.len();

            if branchings == 0 && epsilons == 0 {
                result.push(candidate);
                continue;
            }

            if branchings == 0 && epsilons == 1 {
                candidate.id = state.epsilons[0];
                candidates.push(candidate);
                continue;
            }

            if branchings == 1 && epsilons == 0 {
                // If the candiadte state can only be followed by a single u8, then
                // push that u8 onto the candidate.

                for (index, b) in byteclass.iter().enumerate() {
                    if *b != 0 {
                        // This test might break in the future if we do a unlikly optimisation
                        // were the first entry in state.table is not empty.
                        // @TODO consider what to do with error case
                        candidate
                            .push_byte(index as u8, candidate.id.clone())
                            .unwrap();
                        let next_stateid = state.table[*b as usize];
                        candidate.id = next_stateid;
                        candidates.push(candidate);
                        break;
                    }
                }
                continue;
            }

            // From here onwards we have branching
            candidate.branch_depth += 1;

            // This way of limiting branching is slighly dumb, but works. We now limit branching on the
            // byte level and not the char level. That means that if you have a branching limit of 5, techincally
            // we can branch into 5^2 or 25 different chars. @TODO fix this by tracking some info in Candidate.
            if (BRANCH_LIMIT as usize) < (branchings as usize) + (epsilons as usize) {
                result.push(candidate);
                continue;
            }

            if BRANCH_DEPTH <= candidate.branch_depth {
                // Then we dont continue branching
                result.push(candidate);
                continue;
            }

            // We add epsilons to candidate stack
            for epsilon in &state.epsilons {
                let mut new_cand = candidate.clone();
                new_cand.id = *epsilon;
                candidates.push(new_cand);
            }

            for (index, b) in byteclass.iter().enumerate() {
                if *b != 0 {
                    // This test might break in the future if we do a unlikly optimisation
                    // were the first entry in state.table is not empty.
                    let neighbour = &state.table[*b as usize];
                    let mut new_cand = candidate.clone();
                    //@TODO consider what to do with error case.
                    new_cand.push_byte(index as u8, *neighbour).unwrap();
                    candidates.push(new_cand);
                }
            }
        }

        result.into_iter().map(|cand| cand.string).collect()
    }
}

// #[cfg(test)]
// mod test {
//     use std::iter;

//     use crate::automata::{state::StateId, NFA};

//     #[test]
//     fn literal_tab_complete() {
//         let nfa = NFA::literal("Hello").unwrap();
//         let tabs = nfa.tab_complete(iter::once(StateId::of(0)).collect());
//         assert!(tabs.len() == 1);
//         assert!(tabs[0] == "Hello");
//     }

//     #[quickcheck]
//     fn qc_literal_tab_complete(string: String) -> bool {
//         let nfa = NFA::literal(string.as_str()).unwrap();
//         let tabs = nfa.tab_complete(iter::once(StateId::of(0)).collect());
//         tabs.len() == 1 && tabs[0] == string
//     }
// }
