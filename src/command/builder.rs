use super::command::CommandSpec;
use crate::{
    argument::Argument,
    generic::Func,
    parser::{self, And, IterParser, MaybeSpaces, OneOrMoreSpace},
};

// use std::marker::PhantomData;
pub fn space() -> OneOrMoreSpace {
    OneOrMoreSpace
}

pub fn literal(value: &str) -> parser::Literal {
    parser::Literal {
        value: value.to_string(),
    }
}

pub trait CommandBuilder {
    type Parser: IterParser;
    fn arg<A: Argument>(self) -> And<Self::Parser, <A as Argument>::Parser>;
    fn space(self) -> And<Self::Parser, OneOrMoreSpace>;
    fn followed_by<P: IterParser>(self, parser: P) -> And<Self::Parser, P>;
    fn on_call<GameState, CommandResult, F1, F2>(
        self,
        f: F1,
    ) -> CommandSpec<GameState, CommandResult, F1, F2, And<Self::Parser, MaybeSpaces>>
    where
        F1: Func<<Self::Parser as IterParser>::Extract, Output = F2>,
        F2: Func<GameState, Output = CommandResult>;
}

impl<T> CommandBuilder for T
where
    T: IterParser,
{
    type Parser = T;

    fn arg<A: Argument>(self) -> And<Self::Parser, A::Parser> {
        And {
            a: self,
            b: A::Parser::default(),
        }
    }

    fn followed_by<P: IterParser>(self, other: P) -> And<Self::Parser, P> {
        And { a: self, b: other }
    }

    fn space(self) -> And<Self::Parser, OneOrMoreSpace> {
        self.followed_by(space())
    }

    fn on_call<GameState, CommandResult, F1, F2>(
        self,
        f: F1,
    ) -> CommandSpec<GameState, CommandResult, F1, F2, And<Self::Parser, MaybeSpaces>>
    where
        F1: Func<<Self::Parser as IterParser>::Extract, Output = F2>,
        F2: Func<GameState, Output = CommandResult>,
    {
        CommandSpec {
            parser: self.followed_by(MaybeSpaces),
            mapping: f,
            gamestate: Default::default(),
            command_result: Default::default(),
            mapping_result: Default::default(),
        }
    }
}
