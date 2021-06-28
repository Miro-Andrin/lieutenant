use crate::IterParser;

// A argument is just  a type that has a default way of parsing it.

pub trait Argument {
    type World;
    type Parser : IterParser<Self::World> + Default + Sized;
}


