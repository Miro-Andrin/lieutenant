use anyhow::bail;

use super::{Parser, Result};
use crate::{
    generic::{Combine, CombinedTuples, Tuple},
};

use std::fmt;

pub struct Space;

impl Parser for Space {
    type Extract = ();

    #[inline]
    fn parse(&self, _input: &mut &str) -> Result<Self::Extract> {
        Ok(())
    }
}

impl fmt::Debug for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;
        Ok(())
    }
}
