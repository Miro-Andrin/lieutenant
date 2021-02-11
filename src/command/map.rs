use anyhow::Result;

use crate::generic::Func;

use super::Parser;

pub struct Map<P, F> {
    pub(crate) parser: P,
    pub(crate) map: F,
}

impl<P, F> Parser for Map<P, F>
where
    P: Parser,
    F: Func<P::Extract> 
{
    type Extract = (F::Output,);

    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let args = self.parser.parse(input)?;
        Ok((self.map.call(args),))
    }
}