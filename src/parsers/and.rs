use regex_syntax::hir::Hir;

use super::{Parser, Result};
use crate::generic::{Tuple, Combine, CombinedTuples};

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
    fn parse(&self, input: &[&str]) -> Result<Self::Extract> {
        let a = self.a.parse(input)?;
        let b = self.b.parse(input)?;

        Ok(a.combine(b))
    }

    #[inline]
    fn regex(&self, syntax: &mut Vec<Hir>) {
        self.a.regex(syntax);
        self.b.regex(syntax);
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