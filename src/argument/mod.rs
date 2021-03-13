mod numbers;
use crate::parser::parser::IterParser;
pub trait Argument<S> {
    type Parser: IterParser<Extract = (Self,), ParserState = S>;
}
