pub mod command;
pub mod error;
pub(crate) mod generic;
pub mod values;
pub mod graph;
pub mod automaton;

pub use command::*;
pub use error::*;

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::automaton::{NFA, DFA, dfa, StateId, ByteClass};
    use std::mem;

    #[test]
    fn test() {
        let command = BlankBuilder::new()
            .literal("tp")
            .param()
            .param()
            .param()
            .build::<(), _>(|_x: i32, _y: i32, _z: i32| Ok(()));
        
        let nfa = NFA::from(command);

        let dfa = DFA::from(nfa);

        let mut size = mem::size_of::<DFA>();
        size += mem::size_of::<dfa::State>() * dfa.states.len();
        size += dfa.states.iter().map(|state| state.table.len() * mem::size_of::<StateId>()).sum::<usize>();
        size += dfa.classes.len() * mem::size_of::<ByteClass>();
        size += dfa.classes.iter().map(|class| class.0.len() * mem::size_of::<u8>()).sum::<usize>();

        println!("size before: {}", size);

        let dfa = dfa.minimize();
        println!("size after: {}", size);

        assert!(dfa.find("tp 1 1 1").is_some());
    }
}