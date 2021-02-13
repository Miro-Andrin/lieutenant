use anyhow::bail;

use super::{Parser, Result};
use crate::{
    generic::{Combine, CombinedTuples, Tuple},
    AddToDispatcher, Dispatcher, NodeId,
};

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
    /// Note: On failure the input can be left in a bad state. So if you need to use the input after an
    /// error its your responisbillity to have stored a copy.
    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let a = self.a.parse(input)?;
        println!("AND HERE WE GO 1");

        // Removes at least one space.
        let prev_len = input.len();
        *input = input.trim_start();
        if input.len() == prev_len {
            bail!("")
        }
        
        println!("AND HERE WE GO 2");

        let b = self.b.parse(input)?;

        println!("AND HERE WE GO 3");

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
        write!(f, " ")?;
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

#[cfg(test)]
mod tests {
    use crate::command::Literal;

    use super::*;

    #[test]
    fn simple() {
        let lit1 = Literal {
            value: String::from("tp"),
        };

        let lit2 = Literal {
            value: String::from("tp"),
        };

        let x = And { a: lit1, b: lit2 };

        let input = &mut "tp tp a";

        assert!(x.parse(input).is_ok());

        assert_eq!(input, &"a");
    }

    #[test]
    fn simple2() {
        let lit1 = Literal {
            value: String::from("tp"),
        };

        let lit2 = Literal {
            value: String::from("tp"),
        };

        let x = And { a: lit1, b: lit2 };

        let input = &mut "tp tp";

        assert!(x.parse(input).is_ok());

        assert_eq!(input, &"");
    }

    #[test]
    fn simple3() {
        let lit1 = Literal {
            value: String::from("tp"),
        };

        let lit2 = Literal {
            value: String::from("tp"),
        };

        let x = And { a: lit1, b: lit2 };

        let input = &mut "tptp";

        assert!(x.parse(input).is_err());

        //
        assert_eq!(input, &"tp");
    }
}
