use std::marker::PhantomData;

use anyhow::bail;

use crate::{generic::Func, parser::IterParser};

#[derive(Clone, Copy, Default, PartialEq, Eq, std::hash::Hash, Debug)]
pub struct CommandId {
    pub(crate) id: usize,
}

impl CommandId {
    pub fn of(value: usize) -> Self {
        Self { id: value }
    }
}

pub trait Command<GameState, Res> {
    fn call(&self, gamestate: GameState, input: &str) -> Result<Res, GameState>;

    fn regex(&self) -> String;
}

pub struct CommandSpec<P, GameState, F> {
    parser: P,
    game_state: PhantomData<GameState>,
    f: PhantomData<F>,
}

impl<P,GameState,F> CommandSpec<P,GameState,F> {
    pub fn new(parser: P) -> CommandSpec<P,GameState,F>{
        Self {
            parser,
            game_state: Default::default(),
            f: Default::default(),
        }
    }
}

impl<Res, P: IterParser, GameState, F> Command<GameState, Res> for CommandSpec<P, GameState, F>
where
    
    F: Func<GameState, Output = Res>,
    P: IterParser<Extract = (F,)>,
{
    fn regex(&self) -> String {
        self.parser.regex()
    }

    fn call(&self, gamestate: GameState, input: &str) -> Result<Res, GameState> {
        let mut state = P::ParserState::default();
        loop {
            match self.parser.parse(state, input) {
                (Ok(((func,), _)), _) => return Ok(func.call(gamestate)),
                (Err(_), None) => {
                    //bail!("Not able to parse input");
                    return Err(gamestate);
                }
                (Err(_), Some(next_state)) => state = next_state,
            }
        }
    }
}
