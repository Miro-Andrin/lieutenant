pub enum CommandError {
    Parse { msg: String },
    Exec { msg: String },
}
pub type CommandResult = Result<u64, CommandError>;

pub trait Command: Sized + Eq {
    fn call<'p>(&self, input: &'p str) -> CommandResult;
    fn regex(&self) -> String;
}

