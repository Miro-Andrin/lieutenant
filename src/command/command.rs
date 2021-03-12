use std::marker::PhantomData;

use anyhow::bail;

use crate::parser::Parser;


#[derive(Clone,Copy,Default,PartialEq, Eq,std::hash::Hash,Debug)]
pub struct CommandId {
    pub(crate) id : usize
}

impl CommandId {
    pub fn of(value: usize) -> Self {
        Self {
            id :value
        }
    }
}

pub trait Command<GameState, Res> {
    fn call<'a>(&self, game_state: &GameState, input: &str) -> anyhow::Result<Res>;
    fn regex(&self) -> String;
}

struct CommandSpec<P, GameState, Res, A>
{
    parser: P,
    game_state: PhantomData<GameState>,
    res: PhantomData<Res>,
    a: PhantomData<A>
}


impl<GameState, Res, P, A> Command<GameState, Res> for CommandSpec<P, GameState, Res, P::Extract>
where
    A: Fn(&GameState) -> Res,
    P: Parser<Extract = (A,)>,
    {
    fn call(&self, game_state: &GameState, input: &str) -> anyhow::Result<Res> {
        let mut state = P::ParserState::default();

        loop {
            match self.parser.parse(state, input) {
                (Ok(((func,), _  )), _) => {
                    return Ok(func(game_state))
                }
                (Err(_), None) => {
                    bail!("Not able to parse input");
                }
                (Err(_), Some(next_state)) => {
                    state = next_state
                }
            }
        }
    }

    fn regex(&self) -> String {
            self.parser.regex()
    }
}
