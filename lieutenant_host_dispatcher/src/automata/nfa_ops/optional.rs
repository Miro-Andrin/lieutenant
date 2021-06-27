use crate::automata::{NFA, state::StateId};


impl NFA {
    
    pub(crate) fn make_optional(&mut self) {
        for end in self.ends.clone() {
            self.push_epsilon(StateId::of(0), end)
        }
    }

}