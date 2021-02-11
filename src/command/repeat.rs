use std::todo;

use super::{Parser, Result};

pub struct Repeat<P> {
    parser: P,
}

impl<P> Parser for Repeat<P>
where
    P: Parser,
{
    type Extract = (Vec<P::Extract>,);

    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        todo!()
    }
}

pub fn repeat<P>(parser: P) -> Repeat<P> {
    Repeat { parser }
}
