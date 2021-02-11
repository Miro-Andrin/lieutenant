mod and;
mod literal;
mod space;
mod argument;
mod repeat;
mod map;

use std::{str::FromStr};

pub use and::*;
pub use argument::*;
pub use literal::*;
pub use space::*;
pub use repeat::*;
pub use map::*;

use crate::{AddToDispatcher, Dispatcher, NodeId, generic::{Func, Tuple}};
use crate::Result; 

pub trait Parser {
    type Extract: Tuple;

    fn parse(&self, input: &mut &str) -> Result<Self::Extract>;
}

pub struct Command<P, F> {
    pub(crate) parser: P,
    pub(crate) callback: F,
}

impl<P, F> Command<P, F>
where
    P: Parser,
    F: Func<P::Extract>
{
    pub fn call(&self, mut input: &str) -> Result<F::Output> {
        let args = self.parser.parse(&mut input)?;
        Ok(self.callback.call(args))
    }
}

pub trait CommandBuilder: Parser + Sized {
    fn and<T: Parser>(self, other: T) -> And<Self, T> {
        And { a: self, b: other }
    }
    
    fn arg<T: FromStr>(self) -> And<Self, Argument<T>> {
        self.and(argument())
    }

    fn map<F: Func<Self::Extract>>(self, map: F) -> Map<Self, F> {
        Map { parser: self, map }
    }

    fn build<F: Func<Self::Extract>>(self, callback: F) -> Command<Self, F> {
        Command { parser: self, callback }
    }
}

impl<T> CommandBuilder for T where T: Parser + Sized {}

impl<P, F> AddToDispatcher for Command<P, F>
where
    P: AddToDispatcher
{
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        let node_id = self.parser.add_to_dispatcher(parent, dispatcher);
        node_id
    }
}