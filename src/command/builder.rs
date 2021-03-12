use crate::{generic::Tuple, parser::{MaybeSpaces, Parser}};

pub struct CommandBuilder<P> {
    parser: Option<P>,
}

impl<P: Parser> CommandBuilder<P> {

    // Matches nothing
    pub fn new<A>() -> CommandBuilder<A>{
        CommandBuilder {
            parser: None
        }
    }

    pub fn followed_by<OP: Parser>(parser: OP) -> Self<> {

    }

    

}