use core::fmt;

use crate::generic::Tuple;

pub trait Parser {
    type Extract: Tuple;
    type State: Default + std::fmt::Debug;

    fn parse<'p>(
        &self,
        state: Self::State,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::State>,
    );

    /// This method should return a regex that recognises a language that is a superset of what the parser recognises.
    /// Another way to put it. If the parser sucsessfully parses some input, then the regex should have matched the part
    /// that it consumed, but it does not have to be the other way arround. Theoretically we could therefor always use '.*?'
    /// as the regex, but then its not as usefull.
    /// We use the regex as a heuristic to determine if a parser can parse some input. So if for example you expect a parser
    /// to be able to parse json then a suitable regex would be "\{.*?\}". Using this regex we can reduce the need to go from
    /// feather core to the wasm instance the plugin is running in.
    fn regex(&self) -> String;
}
