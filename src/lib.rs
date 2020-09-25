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
    use crate::automaton::{NFA, DFA};

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

        assert!(dfa.find("tp 1 1 1").is_some());
    }
}