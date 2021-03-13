use crate::{argument::Argument, generic::{Combine, Func, Tuple}, parser::{self, And, IterParser, Map}};

use super::command::{self, CommandSpec};

pub struct CommandBuilder<P: IterParser> {
    parser: P,
}

impl<P: IterParser> CommandBuilder<P> {
    pub fn new<A: IterParser>() -> CommandBuilder<parser::Literal> {
        CommandBuilder {
            parser: parser::Literal::new("".to_string()),
        }
    }

    pub fn arg<S: Default, T: Argument<S>>(self) -> CommandBuilder<And<P, impl IterParser>>
    where
        <T as Argument<S>>::Parser: Sized + Default,
        <P::Extract as Tuple>::HList:
        Combine<<<T::Parser as IterParser>::Extract as Tuple>::HList>,
    {
        Self {
            parser: And {
                a: self.parser,
                b: And {
                    a: parser::OneOrMoreSpace::new(),
                    b: T::Parser::default(),
                },
            }
        }
    }

    pub fn opt_arg<S: Default, T: Argument<S>>(self) ->  CommandBuilder<And<P, impl IterParser>>
    where
        <T as Argument<S>>::Parser: Sized + Default,
    {   
        Self{
            parser: And {
            a: self.parser,
            b: parser::Opt {
                parser: parser::And {
                    a: parser::OneOrMoreSpace::new(),
                    b: T::Parser::default(),
                },
            },
        }
    }
    }

    pub fn map<F>(self, map: F) -> Map<P, F>
    where
        F : Func<P::Extract>,
    {
        parser::Map {
            parser: self.parser,
            map,
        }
    }

    pub fn build<R,Res, F: Func<P::Extract,Output = Res>,GameState>(self, callback: F) ->  CommandSpec<P, GameState, F> {
        let mapped = self.map(callback);
        return CommandSpec::new(mapped.parser);
    }
}
