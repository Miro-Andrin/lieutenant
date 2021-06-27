use std::iter;

use crate::automata::AutomataBuildError;
use crate::automata::NFA;

/*
This can be optimised by generating a different nfa if self.ends.len() == 1.
If that is the case we know
*/
impl NFA {
    /// Zero or more matches.
    /// NOTE: Does not preserve assosiated values!
    pub fn repeat(self) -> Result<Self, AutomataBuildError> {
        //    /–>––––––––––––––––––––––––––>\
        // >(a) -> (b) -> [self] -> (c) ->((d))
        //          \<-------------</

        if self.is_empty() {
            return Ok(self);
        }

        let mut nfa = NFA::with_capacity(
            self.states.len() + 4,
            self.translations.len(),
            self.ends.len(),
        );

        let a = nfa.push_state();
        let b = nfa.push_state();
        nfa.push_epsilon(a, b);

        // b -> [self]
        let (self_start, self_ends) = nfa.extend(self, nfa.states.len())?;
        nfa.push_epsilon(b, self_start);

        let c = nfa.push_state();
        for e in self_ends {
            nfa.push_epsilon(e, c);
        }

        let d = nfa.push_state();

        nfa.push_epsilon(c, b);
        nfa.push_epsilon(c, d);
        nfa.push_epsilon(a, d);

        nfa.ends = iter::once(d).collect();

        Ok(nfa)
    }
}

#[cfg(test)]
mod test {

    fn is_match(nfa: &NFA, res: Result<BTreeSet<StateId>, BTreeSet<StateId>>) -> bool {
        match res {
            Ok(x) => x.iter().any(|x| nfa.is_end(x)),
            Err(_x) => false,
        }
    }

    use std::collections::BTreeSet;

    use crate::automata::state::StateId;

    use super::*;

    #[test]
    fn repeat() {
        let testcase = ["a", "b", "ab", "", " "];
        for t in testcase.iter() {
            let head_nfa = NFA::literal(t).unwrap();
            let nfa = head_nfa.repeat().unwrap();

            assert!(nfa.find(format!("").as_str()).is_ok(), "{}", t);
            assert!(nfa.find(t.to_string().as_str()).is_ok(), "{} one", t);
            assert!(
                nfa.find(format!("{}{}", t, t).as_str()).is_ok(),
                "{} two",
                t
            );

            assert!(
                nfa.find(format!("{}{}{}", t, t, t).as_str()).is_ok(),
                "{} three",
                t
            );
        }
    }

    #[quickcheck]
    fn qc_literal_find_failure(a: String, b: String) -> bool {
        let nfa = NFA::literal(a.as_str()).unwrap().repeat().unwrap();

        if b.is_empty() || a.is_empty() {
            return true;
        }

        let mut rest = b.as_str();
        while rest.starts_with(&a) {
            rest = &rest[a.len()..];
            if rest.is_empty() {
                // b is a repeated multiple times
                return true;
            }
        }

        match nfa.find(a) {
            Ok(x) => {
                if x.iter().all(|x| !nfa.is_end(x)) {
                    // Then find did not result in a end state
                    return false;
                }
            }
            Err(_) => return false,
        }

        match nfa.find(b) {
            Ok(x) => {
                if x.iter().all(|x| !nfa.is_end(x)) {
                    // Then find did not result in a end state
                    return true;
                } else {
                    return false;
                }
            }
            Err(_x) => true,
        }
    }

    #[quickcheck]
    fn a_or_b_repeat(a: String, b: String) -> bool {
        let a_nfa = NFA::literal(&a).unwrap();
        let b_nfa = NFA::literal(&b).unwrap();

        let or = a_nfa.union(b_nfa).unwrap().repeat().unwrap();

        // empty string match
        let res = or.find("");
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(a.as_str());
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(b.as_str());
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}", a, a));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}", a, a, a));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}", b, b));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}", b, b, b));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}", a, b));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}", a, b, a));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}", a, b, b));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}{}", a, b, b, a));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}{}", a, b, a, a));
        if !is_match(&or, res) {
            return false;
        }

        let res = or.find(format!("{}{}{}{}", a, b, a, b));
        if !is_match(&or, res) {
            return false;
        }

        true
    }
}
