use crate::automata::{AutomataBuildError, NFA};

impl NFA {
    pub fn union(self, other: NFA) -> Result<Self, AutomataBuildError> {
        //    /–>––[self]
        // >(a)
        //   \–>––[other]

        // If self or other contains no states then or'ing it is a no-op.
        if other.is_empty() {
            return Ok(self);
        } else if self.is_empty() {
            return Ok(other);
        }

        let mut nfa = NFA::with_capacity(
            self.states.len() + other.states.len() + 1,
            self.translations.len() + other.translations.len(),
            self.ends.len() + other.ends.len(),
        );

        /*
            In this method there is probably  opertunities to realy improve the generated nfa, such that
            there is less to optimise about it. 

        */

        let a = nfa.push_state();
        let (self_start, self_ends) = nfa.extend(self, nfa.states.len())?;
        nfa.push_epsilon(a, self_start);

        let (other_start, other_ends) = nfa.extend(other, nfa.states.len())?;
        nfa.push_epsilon(a, other_start);
        nfa.ends.extend(other_ends.into_iter());
        nfa.ends.extend(self_ends.into_iter());

        // Make the assosiated values of a (the new start state) the union of the two previus start states.
        let ass = &nfa[self_start].assosiations.clone();
        let bss = &nfa[other_start].assosiations.clone();
        nfa[a].assosiations.union_with(ass);
        nfa[a].assosiations.union_with(bss);

        return Ok(nfa);
    }
}

#[cfg(test)]
mod test {
    use crate::automata::{NFA, state::StateId};

    #[test]
    fn test_union() {
        let a = NFA::literal("Aa").unwrap();
        let b = NFA::literal("Bb").unwrap();
        let either = a.union(b).unwrap();

        assert!(either.find("Aa").is_ok());
        assert!(either.find("Bb").is_ok());
    }


    #[test]
    fn test_assosiated_values() {
        let mut a = NFA::literal("Abc").unwrap();
        let mut b = NFA::literal("Ade").unwrap();

        a.assosiate_with(0);
        b.assosiate_with(1);

        let c = a.union(b).unwrap();

        assert!(c[StateId::of(0)].assosiations.contains(0));
        assert!(c[StateId::of(0)].assosiations.contains(1));

    }
}

