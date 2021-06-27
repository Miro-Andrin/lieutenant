mod generics;
mod parsers;
mod impl_macro;
mod builder;
mod argument;

use generics::Tuple;
pub use builder::*;
pub use parsers::*;

#[derive(Debug, Clone)]
pub struct ParseError<'p> {
    /// What parts of the input were we not able to parse
    rest: &'p str,
    /// An explenation of what happend.
    msg: String,
}

impl<'p> ParseError<'p> {
    fn got_further(&self, other: &Option<Self>) -> bool {
        match other {
            None => true,
            Some(other) => self.rest.len() < other.rest.len(),
        }
    }
}

// pub trait Parser{
//     type World;
//     type Extract: Tuple;
//     fn parse<'p>(&self, world: &Self::World, input: &'p str) -> Result<(Self::Extract, &'p str),ParseError<'p>>;
//     fn regex(&self) -> String;
// }


pub trait IterParser<World> : PartialEq + Eq{
    type State: Default + Clone;
    type Extract: Tuple;

    /**
        Generator like interface for parsing. The reason this is not just
        ```rust
            fn parse<'p>(&self, input: &'p str) ->
                Result<(Self::Extract, &'p str), ParseError>
        ```
        is because for arguments like Option<u32>, we sometimes need to do backtracking
        and attempt to parse the argument as either Some or None.

        The output state of one call to parse is the input to the next call, unless None is
        returned wich signals that the parser has exhausted all its different parsing attempts.
        The state starts out as Default::default().
    */
    fn iter_parse<'p>(
        &self,
        world: &World,
        state: Self::State,
        input: &'p str,
    ) -> (
        Result<(Self::Extract, &'p str), ParseError<'p>>,
        Option<Self::State>,
    );

    fn regex(&self) -> String;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
