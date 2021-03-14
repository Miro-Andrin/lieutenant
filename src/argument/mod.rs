mod numbers;
pub use numbers::*;
use crate::parser::parser::IterParser;

pub trait Argument<S> {
    type Parser: IterParser<Extract = (Self,), ParserState = S> + Sized + Default;
}
