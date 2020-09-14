use super::{Func, Input, Parser, ParserBase, Result};

#[derive(Clone)]
pub struct Map<P, F> {
    pub(super) parser: P,
    pub(super) callback: F,
}

impl<P, F> ParserBase for Map<P, F>
where
    P: Parser,
    F: Func<P::Extract>,
{
    type Extract = (F::Output,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let ex = self.parser.parse(input)?;
        Ok((self.callback.call(ex),))
    }
}