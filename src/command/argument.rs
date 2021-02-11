use std::{fmt, marker::PhantomData, str::FromStr, todo};

use anyhow::{anyhow, bail};

use crate::{AddToDispatcher, Dispatcher, Node, NodeId, Validator};

use super::{Parser, Result};

#[derive(Debug)]
pub struct Argument<A> {
    pub(crate) argument: PhantomData<A>,
}

impl<A> Clone for Argument<A> {
    fn clone(&self) -> Self {
        Self { argument: PhantomData::default() }
    }
}

impl<A> Parser for Argument<A>
where
    A: FromStr,
{
    type Extract = (A,);

    #[inline]
    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let arg = A::from_str(&input).map_err(|_| anyhow!("could not parse argument"))?;

        if input.is_empty() {
            bail!("input is empty!");
        }

        Ok((arg,))
    }
}

pub fn argument<A>() -> Argument<A>
where
    A: FromStr,
{
    Argument {
        argument: Default::default(),
    }
}

impl<T> fmt::Display for Argument<T>
where
    T: Integer
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<int>")?;
        Ok(())
    }
}

pub trait Integer: FromStr {}

impl Integer for u8 {}
impl Integer for i8 {}
impl Integer for u16 {}
impl Integer for i16 {}
impl Integer for u32 {}
impl Integer for i32 {}
impl Integer for u64 {}
impl Integer for i64 {}
impl Integer for u128 {}
impl Integer for i128 {}

impl<T: FromStr> Validator for Argument<T> {
    fn validate(&self, input: &mut &str) -> bool {
        T::from_str(*input).is_ok()
    }
}

impl<T: FromStr + 'static> AddToDispatcher for Argument<T>  {
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        dispatcher.add(parent, Node::new(self.clone()))
    }
}
