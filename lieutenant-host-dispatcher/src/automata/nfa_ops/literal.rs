use std::iter;

use crate::automata::{AutomataBuildError, NFA};

impl NFA {
    // Takes a string and constructs a nfa that recognises the string. By that i mean 
    // that calling '.find' on it returns Ok. 
    pub fn literal(lit: &str) -> Result<Self, AutomataBuildError> {
        let mut nfa = NFA::with_capacity(lit.len(), lit.len(), 1);
        let mut prev = nfa.push_state();

        for c in lit.bytes() {
            let next = nfa.push_state();
            nfa.push_connection(prev, next, c)?;
            prev = next;
        }

        nfa.ends = iter::once(prev).collect();
        Ok(nfa)
    }
}

#[cfg(test)]
mod test {
    use crate::automata::NFA;

    #[test]
    fn literal_empty_str() {
        let a = NFA::literal("").unwrap();
        assert!(a.find("").is_ok())
    }

    #[test]
    fn literal_test_simple() {
        let a = NFA::literal("Abc").unwrap();
        assert!(a.states.len() == 4);
    }

    #[quickcheck]
    fn qc_literal_1(a: String) -> bool {
        let nfa = NFA::literal(a.as_str()).unwrap();
        nfa.states.len() == (a.len() + 1)
    }

    #[quickcheck]
    fn qc_literal_find_sucsess(a: String) -> bool {
        let nfa = NFA::literal(a.as_str()).unwrap();
        match nfa.find(a) {
            Ok(set) => set.len() == 1,
            Err(_) => false,
        }
    }

    #[quickcheck]
    fn qc_literal_find_failure(a: String, b: String) -> bool {
        let nfa = NFA::literal(a.as_str()).unwrap();
        nfa.find(b.as_str()).is_ok() == (a == b)
    }
}
