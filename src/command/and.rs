use super::{Parser, Result};
use crate::{AddToDispatcher, Dispatcher, Node, NodeId, generic::{Tuple, Combine, CombinedTuples}};

use std::fmt;

pub struct And<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> Parser for And<A, B>
where
    A: Parser,
    <<A as Parser>::Extract as Tuple>::HList: Combine<<<B as Parser>::Extract as Tuple>::HList>,
    B: Parser,
{
    type Extract = CombinedTuples<A::Extract, B::Extract>;

    #[inline]
    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let a = self.a.parse(input)?;
        let b = self.b.parse(input)?;

        Ok(a.combine(b))
    }
}

impl<A, B> fmt::Display for And<A, B>
where
    A: fmt::Display,
    B: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.a.fmt(f)?;
        self.b.fmt(f)?;
        Ok(())
    }
}

impl<A, B> AddToDispatcher for And<A, B>
where
    A: AddToDispatcher,
    B: AddToDispatcher,
{
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        let parent = self.a.add_to_dispatcher(parent, dispatcher);
        self.b.add_to_dispatcher(Some(parent), dispatcher)
    }
}