use crate::{IterParser, ParseError};

#[derive(Clone, Copy, PartialEq, Eq)]
/// One or more space
pub struct OneOrMoreSpace;

impl<World> IterParser<World> for OneOrMoreSpace {
    type State = ();
    type Extract = ();

    fn iter_parse<'p>(
        &self,
        _world: &World,
        _state: Self::State,
        input: &'p str,
    ) -> (
        Result<(Self::Extract, &'p str), crate::ParseError<'p>>,
        Option<Self::State>,
    ) {
        
        if input.len() == 0 {
            return (
                Err(
                    ParseError {
                        rest: input,
                        msg: "Expected to find a space, but input was empty".to_owned(),
                    }
                )
                ,
                None
            )
        }

        let trimemd = input.trim_start();

        if trimemd.len() == input.len() {
            return (
                Err(
                    ParseError {
                        rest: input,
                        msg: format!("Expected to find a space, but found the char {}",input.chars().nth(0).unwrap()),
                    }
                )
                ,
                None
            )
        }

        (
            Ok((
                (),
                trimemd
            )),
            None
        )
    }

    fn regex(&self) -> String {
        r"\s+".to_owned()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Zero or more space
pub struct MaybeSpaces;




impl<Wold> IterParser<Wold> for MaybeSpaces {
    type State = ();
    type Extract = ();

    fn iter_parse<'p>(
        &self,
        _world: &Wold,
        _state: Self::State,
        input: &'p str,
    ) -> (
        Result<(Self::Extract, &'p str), crate::ParseError<'p>>,
        Option<Self::State>,
    ) {
        let trimemd = input.trim_start();
        (
            Ok((
                (),
                trimemd
            )),
            None
        )
    }

    fn regex(&self) -> String {
        r"\s*".to_owned()
    }
}