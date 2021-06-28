use std::marker::PhantomData;

use crate::{
    argument::Argument,
    generics::Func,
    parsers::{And, MaybeSpaces, OneOrMoreSpace, Opt},
    IterParser, ParseError,
};


pub struct CommandSpec<Game, CmdRes, F1, F2, P>
{
    pub parser: P,
    pub(crate) mapping: F1,
    pub(crate) gamestate: PhantomData<Game>,
    pub(crate) command_result: PhantomData<CmdRes>,
    pub(crate) mapping_result: PhantomData<F2>,
}

impl<Game, CmdRes, F1, F2, P> PartialEq for CommandSpec<Game, CmdRes, F1, F2, P>
where
    P: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.parser.eq(&other.parser)
    }
}

impl<Game, CmdRes, F1, F2, P> Eq for CommandSpec<Game, CmdRes, F1, F2, P> where P: Eq {}


impl<Game, CmdRes, F1, F2, P> CommandSpec<Game, CmdRes, F1, F2, P>
where
    P: IterParser<Game>,
    F1: Func<<P as IterParser<Game>>::Extract, Output = F2>,
    F2: Func<Game, Output = CmdRes>,
{
   pub fn call<'p>(&self, game: Game, input: &'p str) -> Result<CmdRes, ParseError<'p>> {
        let mut state = Default::default();
        let mut best_error = None::<ParseError>;

        loop {
            match self.parser.iter_parse(&game, state, input) {
                (Ok((ext, _out)), None) => {
                    // TODO: check out is empty else throw error
                    return Ok(self.mapping.call(ext).call(game));
                }
                (Ok((ext, _out)), Some(_)) => {
                    return Ok(self.mapping.call(ext).call(game));
                }
                (Err(e), None) => {
                    if e.got_further(&best_error) {
                        best_error = Some(e);
                    }
                    return Err(best_error.unwrap())
                },
                (Err(e), Some(s)) => {
                    if e.got_further(&best_error) {
                        best_error = Some(e);
                    }
                    state = s;
                },
            }
        }
    }
}

/// This trait is implemented for every IterParser/Parser and
/// provides a builder pattern for creating commands.
/// The generics get a bit crazy here.
pub trait CommandBuilder<World> {
    type Parser: IterParser<World>;
    fn arg<A: Argument<World = World>>(
        self,
    ) -> And<Self::Parser, And<OneOrMoreSpace, <A as Argument>::Parser, World>, World>;
    fn opt_arg<A: Argument<World = World>>(
        self,
    ) -> And<Self::Parser, Opt<And<OneOrMoreSpace, <A as Argument>::Parser, World>>,World>;
    fn followed_by<P: IterParser<World>>(self, parser: P) -> And<Self::Parser, P,World>;
    fn on_call<Game, CmdRes, F1, F2>(
        self,
        f: F1,
    ) -> CommandSpec<Game, CmdRes, F1, F2, And<Self::Parser, MaybeSpaces, World>>
    where
        F1: Func<<Self::Parser as IterParser<World>>::Extract, Output = F2>,
        F2: Func<Game, Output = CmdRes>;
        
}

impl<T, World> CommandBuilder<World> for T
where
    T: IterParser<World>,
{
    type Parser = T;

    /// Adds an argument with one or more space between it and what came before.
    fn arg<A: Argument<World=World>>(
        self,
    ) -> And<Self::Parser, And<OneOrMoreSpace, <A as Argument>::Parser, World>,World> {
        And::new(self, And::new(OneOrMoreSpace, A::Parser::default()))
    }
    /// Adds an optional argument with one or more space between it and what came before.
    fn opt_arg<A: Argument<World = World>>(
        self,
    ) -> And<Self::Parser, Opt<And<OneOrMoreSpace, <A as Argument>::Parser, World>>,World> {
        And::new(
            self,
            Opt::new(And::new(OneOrMoreSpace, A::Parser::default())),
        )
    }

    /// Note: This does not add spaces between arguments.
    fn followed_by<P: IterParser<World>>(self, parser: P) -> And<Self::Parser, P,World> {
        And::new(self, parser)
    }

    fn on_call<Game, CmdRes, F1, F2>(
        self,
        f: F1,
    ) -> CommandSpec<Game, CmdRes, F1, F2, And<Self::Parser, MaybeSpaces, World>>
    where
        F1: Func<T::Extract, Output = F2>,
        F2: Func<Game, Output = CmdRes>,
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
