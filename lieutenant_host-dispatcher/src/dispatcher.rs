use std::{collections::BTreeSet};

use crate::{
    automata::{AutomataBuildError, NFA},
    command::{Command, CommandError, CommandResult},
};

#[derive(Debug, Clone)]
pub struct Dispatcher<C> {
    commands: Vec<C>,
    nfa: NFA,
}

impl<C> Dispatcher<C>
where
    C: Command,
{
    
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            nfa: NFA::empty(),
        }
    }

    /// The nfa is not left in a bad state on error,
    /// returns false if command was already registerd.
    /// Errors if command failed to build. Usually this happens
    /// when somebody tries to use a regex feature we dont suport.
    fn add(&mut self, command: C) -> Result<bool, AutomataBuildError> {
        if self.commands.iter().any(|c| *c == command) {
            // Then the command is already registerd.
            return Ok(false);
        }
        let regex = command.regex();
        let mut addition: NFA = NFA::regex(&regex)?;
        self.commands.push(command);
        let id = self.commands.len() - 1;
        addition.assosiate_with(id);

        let old = self.nfa.clone();
        let new = old.union(addition)?;
        self.nfa = new;
        Ok(true)
    }

    pub fn remove(&mut self, command: C) -> Result<(), AutomataBuildError> {
        // This command can be made substasially faster by modifying the existing
        // nfa to remove all states only assosiated with the id of the command we are
        // removing. It could even be made O(1) but then it could leave behind some garbage
        // states.

        let id = match self.commands.iter().position(|c| *c == command) {
            Some(id) => id,
            None => {
                // Then command was not pressent in self.commands
                return Ok(());
            }
        };
        
        let removed_command = self.commands.swap_remove(id);

        self.commands.remove(id);
        match self.rebuild() {
            Ok(_) => Ok(()),
            Err(e) => {
                
                // This should never trigger, because a nfa build as per the current
                // code only fails when running out of id's (u32), or when regex are mallformed.
                // Since we know the regex are not mallformed we don't expect this to be a isse. 
                
                self.commands.insert(id, removed_command);
                self.rebuild().unwrap();
                
                Err(e)
            }
        }
    }

    fn rebuild(&mut self) -> Result<(), AutomataBuildError> {
        let mut nfa = NFA::default();

        for (id, command) in self.commands.iter().enumerate() {
            let mut addition = NFA::regex(&command.regex())?;
            addition.assosiate_with(id);
            nfa = nfa.union(addition)?;
        }

        self.nfa = nfa;
        Ok(())
    }

    pub fn dispatch(&self, input: &str) -> CommandResult {

        // Take the input and figure out a small set of 
        // of commands that could potentially be triggerd on this input 
        let states = match self.nfa.find(&input) {
            Ok(x) => x,
            Err(_) => {
                return CommandResult::Err(CommandError::Parse {
                    msg: "No command starts with that.".to_owned(),
                })
            }
        };

        let commands = states
            .into_iter()
            .filter(|x| self.nfa.is_end(x))
            .flat_map(|end| self.nfa[end].assosiations.iter().collect::<Vec<usize>>())
            .collect::<Vec<usize>>();

        if commands.len() == 0 {
            // Then something has gone wrong.
        }

        for id in commands {
            match self.commands[id].call(input) {
                Ok(x) => {
                    return Ok(x);
                }
                Err(CommandError::Parse { msg: _ }) => {
                    // Todo: track the best error and return it if all commands
                    //       fail to parse.
                    continue;
                }
                Err(CommandError::Exec { msg }) => {
                    return Err(CommandError::Exec { msg });
                }
            }
        }

        // print the best error encounterd so far.
        todo!()
    }

    pub fn tab_complete(&self, _input: &str) -> BTreeSet<String> {
        /*
            1) Put input into nfa resulting in a set of commands that
            could start with the given input.

            2) Commands have a option command.regex_tab_completed: bool,
            that determines if a commands tab complete can be done 100%
            with regex, or it needs/wants to handle it itself.
            The reason it might want to handle it itself, is in cases like
            playernames, were the regex won't reflect online players.
            Go to ever plugin and ask it to tab complete those commands.

            3) Do tab complete using the nfa, and merge the results with step two.

        */

        todo!()
    }
}
