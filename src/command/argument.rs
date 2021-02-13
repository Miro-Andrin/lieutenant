use std::{fmt, marker::PhantomData, str::FromStr};

use anyhow::anyhow;

use crate::{regex_validator, AddToDispatcher, Dispatcher, Node, NodeId, Validator};

use super::{OptArgument, Parser, Result};

#[derive(Debug)]
pub struct Argument<A> {
    pub(crate) argument: PhantomData<A>,
}

impl<A> Clone for Argument<A> {
    fn clone(&self) -> Self {
        Self {
            argument: PhantomData::default(),
        }
    }
}

impl<A> Parser for Argument<A>
where
    A: FromStr,
{
    type Extract = (A,);

    #[inline]
    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let mut arg = *input;

        if let Some((i, _)) = input.char_indices().find(|(_, c)| c.is_whitespace()) {
            arg = &input[..i];
            *input = &input[i..];
        }

        let arg = A::from_str(&arg).map_err(|_| anyhow!("could not parse argument"))?;

        Ok((arg,))
    }
}

pub fn argument<A>() -> Argument<A> {
    Argument {
        argument: Default::default(),
    }
}

impl<T: 'static> AddToDispatcher for Argument<T>
where
    Argument<T>: Validator,
{
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        dispatcher.add(parent, Node::new(self.clone()))
    }
}

impl<T> fmt::Display for Argument<T>
where
    T: Integer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<int>")?;
        Ok(())
    }
}

pub trait Integer {}

use lazy_static::lazy_static;

macro_rules! integer {
    [$($ident:ty),*$(,)?] => {
        $(
            regex_validator!(Argument<$ident>, "^[0-9]+");
            impl Integer for $ident {}
        )*
    };
}

integer![u8, i8, u16, i16, u32, i32, u64, i64, u128, i128,];
