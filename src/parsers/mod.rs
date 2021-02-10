mod and;
mod literal;
mod space;
mod argument;
mod repeat;

pub use and::*;
pub use argument::*;
pub use literal::*;
pub use space::*;
pub use repeat::*;

use crate::generic::Tuple;
use crate::Result; 
use regex_syntax::hir::Hir;

pub trait Parser {
    type Extract: Tuple;

    fn parse(&self, input: &[&str]) -> Result<Self::Extract>;
    fn regex(&self, syntax: &mut Vec<Hir>);
}

pub trait Syntax {
    fn regex() -> Hir;
}