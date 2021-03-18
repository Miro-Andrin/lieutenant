use anyhow::bail;

use crate::{
    command::command::Command,
    regex::{dfa::DFA, NFA},
};

use super::command::CommandId;

pub struct Dispatcher<'a, GameState, Res> {
    // Should maybe use a slab, so that we can remove commands, without having to recalculate the dfa.
    commands: Vec<Box<dyn Command<GameState = &'a mut GameState, CommandResult = Res>>>,
    dfa: Option<DFA<CommandId>>,
}

impl<'a, GameState, Res> Dispatcher<'a, GameState, Res> {
    fn build_dfa(&mut self) -> anyhow::Result<()> {
        let mut main_nfa = NFA::<CommandId>::empty();

        for (index, command) in self.commands.iter().enumerate() {
            let aproximation_regex = command.regex();
            let mut command_nfa = NFA::<CommandId>::regex(aproximation_regex.as_str())?;
            command_nfa.assosiate_ends(CommandId::of(index));
            // can fail if we run out of ids, not very likle.
            main_nfa = main_nfa.or(command_nfa)?;
        }

        self.dfa = Some(DFA::from(main_nfa));
        Ok(())
    }

    pub fn call(&self, gamestate: &'a mut GameState, input: &str) -> anyhow::Result<Res> {
        match &self.dfa {
            Some(dfa) => {
                match dfa.find(input) {
                    Ok(id) => {
                        // Todo can probably be a iter or slice
                        let possible_commands = dfa.assosiations(id);
                        //self.commands[0].call(gamestate, input);

                        for command_id in possible_commands {
                            let command = &self.commands[command_id.id];
                            match command.call(gamestate, input) {
                                Ok(x) => return Ok(x),
                                Err(_) => bail!("Failed to parse input") 
                            }
                        }

                        bail!("None of the matching commands was able to parse")
                    }
                    Err(_) => {
                        bail!("No command matched the given input")
                    }
                }
            }
            None => {
                bail!("DFA was not initialized, so no command was able to be called")
            }
        }
    }

    // Is not atomic -.- so if dfa building  fails we are left in a bad state.
    // This can potentially happen if any of the incomming commands use regex features
    // we dont have implemented.
    pub fn submitt_commands<I>(&mut self, commands: I) -> anyhow::Result<()>
    where
        I: IntoIterator<
            Item = Box<dyn Command<GameState = &'a mut GameState, CommandResult = Res>>,
        >,
    {
        self.commands.extend(commands.into_iter());
        self.build_dfa()?;
        Ok(())
    }
}
