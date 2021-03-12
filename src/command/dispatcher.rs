use anyhow::bail;
use indexmap::map::IntoIter;
use regex_syntax::ast::Repetition;

use crate::{command::command::Command, regex::{NFA, dfa::DFA}};

use super::command::CommandId;

struct Dispatcher<GameState,Res> {
    // TODO should maybe use a slab, so that we can remove commands, without having to recalculate the dfa.
    commands: Vec<Box<dyn Command<GameState,Res>>>,
    dfa: Option<DFA::<CommandId>>, 
}

impl<GameState,Res> Dispatcher<GameState,Res> {

    fn build_dfa(&mut self) -> anyhow::Result<()> {

        let mut main_nfa = NFA::<CommandId>::empty();

        for (index, command) in self.commands.iter().enumerate() {
            let aproximation_regex = command.regex();
            let mut command_nfa = NFA::<CommandId>::regex(aproximation_regex.as_str())?;
            command_nfa.assosiate_ends(CommandId::of(index));
            // can fail if we run out of ids, not very likley.
            main_nfa = main_nfa.or(command_nfa)?;
        }   

        self.dfa = Some(DFA::from(main_nfa));
        Ok(())

    }

    pub fn call(&self,state: &GameState, input: &str) -> anyhow::Result<Res> {
        match &self.dfa {
            Some(dfa) => {
                match dfa.find(input) {
                    Ok(id) => {
                        // Todo can probably be a iter or slice 
                        let possible_commands = dfa.assosiations(id);
                        for command_id in possible_commands {
                            let command = &self.commands[command_id.id];
                            if let Ok(res) = command.call(state, input) {
                                return Ok(res)
                            }
                        }
                        bail!("No command matched the input, but some were close");
                    }
                    Err(x) => {
                        bail!("No command matched the input")
                    }
                }
            }
            None => {
                bail!("DFA not initialized, probably means no commands have been submitted.")
            }
        }
    }

    // Is not atomic -.- so if this fails we are left in a bad state. 
    pub fn submitt_commands<I: IntoIterator<Item = Box<dyn Command<GameState,Res>>>>(&mut self,commands : I) -> anyhow::Result<()>{
        self.commands.extend(commands.into_iter());
        self.build_dfa()?;
        Ok(())
    }
}




