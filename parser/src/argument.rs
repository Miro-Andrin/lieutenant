use crate::IterParser;

// A argument is just  a type that has a default way of parsing it.

pub trait Argument<World> {
    type Parser : IterParser<World> + Default + Sized;
}


