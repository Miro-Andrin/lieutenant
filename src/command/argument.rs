use std::{fmt, marker::PhantomData, str::FromStr};

use anyhow::{anyhow, bail};

use crate::{regex_validator, AddToDispatcher, Dispatcher, Node, NodeId, Validator};

use super::{Parser, Result};

#[derive(Debug)]
pub struct Argument<A> {
    pub(crate) argument: PhantomData<A>,
}

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
        println!("{}", input);

        let mut arg = *input;

        if let Some((i, _)) = input.char_indices().find(|(i, c)| c.is_whitespace()) {
            arg = &input[..i];
            *input = &input[i..];
        }

        println!("arg:<{}>",arg);
        println!("input:<{}>",input);

        let arg = A::from_str(&arg).map_err(|_| anyhow!("could not parse argument"))?;

        println!("Duh it works");
        // if input.is_empty() {
        //     bail!("input is empty!");
        // }

        Ok((arg,))
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

pub fn argument<A>() -> Argument<A> {
    Argument {
        argument: Default::default(),
    }
}

pub fn opt_argument<A>() -> OptArgument<A> {
    OptArgument {
        argument: Default::default(),
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
            regex_validator!(OptArgument<$ident>, "^[0-9]+");
            impl Integer for $ident {}
        )*
    };
}

integer![u8, i8, u16, i16, u32, i32, u64, i64, u128, i128,];

impl<T: 'static> AddToDispatcher for Argument<T>
where
    Argument<T>: Validator,
{
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        dispatcher.add(parent, Node::new(self.clone()))
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
