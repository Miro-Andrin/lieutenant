use crate::generic::Tuple;

pub trait IterParser {
    /// This assosiated type says what the return value is for the parser. If you have a parser that returns a i32, then set it to Extract = (i32,), or
    /// if you dont want it returning anythin use Extract = ()
    type Extract: Tuple;

    /// You probably just want to set it to (), and always return None in its place for 'fn parse(&self ...'
    // Internaly we use this for the And, Or and the Optional parsers. This makes implementors of the Parser trait
    // generators of potential parsing results.
    type ParserState: Default;

    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    );

    /// This method should return a regex that recognises a language that is a superset of what the parser recognises.
    /// Another way to put it. If the parser sucsessfully parses some input, then the regex should have matched the part
    /// that it consumed, but it does not have to be the other way arround. Theoretically we could therefor always use '.*?'
    /// as the regex, but then its not as usefull.
    /// We use the regex as a heuristic to determine if a parser can parse some input. So if for example you expect a parser
    /// to be able to parse json then a suitable regex could be "\{.*?\}". Using this regex we can quickly determine what command
    /// a input belongs to.
    fn regex(&self) -> String;
}

trait Parser {
    type Extract;

    fn parse<'p>(&self, input: &'p str) -> (anyhow::Result<(Self::Extract, &'p str)>,);

    fn regex(&self) -> String;

    fn into_iter_parser() {}
}
