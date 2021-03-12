use crate::generic::Func;
use crate::parser::parser::Parser;

pub struct Map<P, F> {
    pub(crate) parser: P,
    pub(crate) map: F,
}
#[derive(Debug)]
pub struct MapState<S> {
    state: S,
}

impl<S: Default> Default for MapState<S> {
    fn default() -> Self {
        MapState {
            state: S::default(),
        }
    }
}

impl<P, F> Parser for Map<P, F>
where
    P: Parser,
    F: Func<P::Extract>,
{
    type Extract = (F::Output,);
    type ParserState = P::ParserState;

    fn parse<'p>(
        &self,
        state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str), anyhow::Error>,
        Option<Self::ParserState>,
    ) {
        let (result, state) = self.parser.parse(state, input);

        match result {
            Ok((ext, out)) => return (Ok(((self.map.call(ext),), out)), state),
            Err(err) => return (Err(err), state),
        }
    }
    fn regex(&self) -> String {
        self.parser.regex()
    }
}
