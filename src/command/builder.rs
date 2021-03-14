use std::marker::PhantomData;

use crate::{
    argument::Argument,
    generic::{Combine, Func, Tuple},
    parser::{self, And, IterParser, Map, OneOrMoreSpace, Opt},
};

use super::command::{CommandSpec};

pub struct CommandBuilder<P: IterParser,G,R> {
    parser: P,
    gamestate: PhantomData<G>,
    result: PhantomData<R>
}

impl<P: IterParser,Res,GameState> CommandBuilder<P,GameState,Res> {
    pub fn new() -> CommandBuilder<parser::Literal,GameState,Res> {
        CommandBuilder::<parser::Literal, GameState,Res> {
            parser: parser::Literal::new("".to_string()),
            gamestate : Default::default(),
            result : Default::default()
        }
    }

    pub fn arg<S: Default, T: Argument<S>>(
        self,
    ) -> CommandBuilder<And<P, And<OneOrMoreSpace, T::Parser>>,GameState,Res>
        
    where
        <P::Extract as Tuple>::HList: Combine<<<T::Parser as IterParser>::Extract as Tuple>::HList>,
        <P as parser::parser::IterParser>::Extract: Clone
    {
        CommandBuilder {
            parser: And {
                a: self.parser,
                b: And {
                    a: parser::OneOrMoreSpace::new(),
                    b: T::Parser::default(),
                },
            },
            gamestate : self.gamestate,
            result : self.result
        }
    }

    pub fn parser<Other: IterParser>(self, it: Other) ->  CommandBuilder<And<P, And<OneOrMoreSpace, Other>>,GameState,Res>
        where
            <P::Extract as Tuple>::HList: Combine<<Other::Extract as Tuple>::HList>,
            <P as parser::parser::IterParser>::Extract: Clone
    {
        CommandBuilder {
            parser: And {
                a: self.parser,
                b: And {
                    a: parser::OneOrMoreSpace::new(),
                    b: it,
                },
            },
            gamestate : self.gamestate,
            result : self.result
        }
    }

    // pub fn opt_arg<S: Default, T: Argument<S>>(
    //     self,
    // ) -> CommandBuilder<And<P, Opt<And<OneOrMoreSpace, T::Parser>>>>
    // where
    //     <P::Extract as Tuple>::HList: Combine<<<T::Parser as IterParser>::Extract as Tuple>::HList>,
    // {
    //     CommandBuilder {
    //         parser: And {
    //             a: self.parser,
    //             b: parser::Opt {
    //                 parser: parser::And {
    //                     a: parser::OneOrMoreSpace::new(),
    //                     b: T::Parser::default(),
    //                 },
    //             },
    //         },
    //     }
    // }

    pub fn map<F>(self, map: F) -> Map<P, F>
    where
        F: Func<P::Extract>,
    {
        parser::Map {
            parser: self.parser,
            map,
        }
    }

    pub fn build<F: Func<P::Extract, Output = Res>>(
        self,
        callback: F,
    ) -> CommandSpec<P, GameState, F> {
        let mapped = self.map(callback);
        return CommandSpec::new(mapped.parser);
    }
}
