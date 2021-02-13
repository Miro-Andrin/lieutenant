use std::marker::PhantomData;

use crate::{AddToDispatcher, Dispatcher, Node, NodeId, Validator};

use super::{argument, Argument, Parser, Result};

#[derive(Debug)]
pub struct OptArgument<A> {
    pub(crate) argument: PhantomData<A>,
}

impl<A> Clone for OptArgument<A> {
    fn clone(&self) -> Self {
        Self {
            argument: PhantomData::default(),
        }
    }
}

impl<A> Parser for OptArgument<A>
where
    Argument<A>: Parser<Extract = (A,)>,
{
    type Extract = (Option<A>,);

    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let original = *input;
        match argument::<A>().parse(input) {
            Ok((result,)) => Ok((Some(result),)),
            Err(_) => {
                *input = original;
                Ok((None,))
            }
        }
    }
}

pub fn opt_argument<A>() -> OptArgument<A> {
    OptArgument {
        argument: Default::default(),
    }
}

impl<T: 'static> AddToDispatcher for OptArgument<T>
where
    OptArgument<T>: Validator,
{
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        dispatcher.add(parent, Node::new(self.clone()))
    }
}

impl<T> Validator for OptArgument<T>
where
    Argument<T>: Validator,
{
    fn validate(&self, input: &mut &str) -> bool {
        let original = *input;
        if !argument::<T>().validate(input) {
            *input = original;
        }

        true
    }
}
