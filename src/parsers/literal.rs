use super::{Parser, Result};
use anyhow::bail;
use regex_syntax::hir::{self, Hir};

use std::{fmt};

/// Matches case ...
pub struct Literal {
    value: String,
}

impl Parser for Literal {
    type Extract = ();

    #[inline]
    fn parse(&self, _input: &mut &[&str]) -> Result<Self::Extract> {
        Ok(())
    }

    #[inline]
    fn regex(&self, syntax: &mut Vec<Hir>) {
        syntax.push(Hir::concat(
            self.value
                .chars()
                .map(|c| Hir::literal(hir::Literal::Unicode(c)))
                .collect(),
        ))
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        Ok(())
    }
}

pub fn literal(literal: &str) -> Literal {
    Literal {
        value: literal.to_owned(),
    }
}
