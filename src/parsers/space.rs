use anyhow::bail;
use regex_syntax::hir::{self, Hir};

use super::{Parser, Result};
use crate::{
    generic::{Combine, CombinedTuples, Tuple},
    CommandBuilder,
};

use std::fmt;

pub struct Space;

impl Parser for Space {
    type Extract = ();

    #[inline]
    fn parse(&self, _input: &mut &[&str]) -> Result<Self::Extract> {
        Ok(())
    }

    #[inline]
    fn regex(&self, syntax: &mut Vec<Hir>) {
        syntax.push(Hir::repetition(hir::Repetition {
            kind: hir::RepetitionKind::OneOrMore,
            greedy: true,
            hir: Box::new(Hir::literal(hir::Literal::Unicode(' '))),
        }));
    }
}

impl fmt::Debug for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;
        Ok(())
    }
}
