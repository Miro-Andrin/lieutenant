mod automata;
mod command;
mod dispatcher;
pub use dispatcher::Dispatcher;
pub use command::{Command, CommandResult, CommandError};

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;


