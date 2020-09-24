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
            .build::<(), _>(|_x: i32| Ok(()));
        
        let nfa = NFA::from(command);
        println!("{:?}", nfa);
        let dfa = DFA::from(nfa);

        assert!(dfa.find("tp 10").is_some());
    }
}