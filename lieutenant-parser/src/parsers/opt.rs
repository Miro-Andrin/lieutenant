use crate::IterParser;

#[derive(PartialEq, Eq,Clone)]
pub struct Opt<A> {
    a: A,
}

impl<A> Opt<A> {
    pub fn new(a: A) -> Self {
        Self { a }
    }
}

#[derive(Debug, Clone)]
pub enum OptState<A> {
    Some(A),
    None,
}

impl<A> Default for OptState<A>
where
    A: Default,
{
    fn default() -> Self {
        Self::Some(A::default())
    }
}

impl<A, World> IterParser<World> for Opt<A>
where
    A: IterParser<World>,
{
    type State = OptState<A::State>;

    type Extract = (Option<A::Extract>,);

    fn iter_parse<'p>(
        &self,
        world: &World,
        state: Self::State,
        input: &'p str,
    ) -> (
        Result<(Self::Extract, &'p str), crate::ParseError<'p>>,
        Option<Self::State>,
    ) {
        match state {
            OptState::Some(a_state) => match self.a.iter_parse(world, a_state, input) {
                (Ok((a_ext, a_out)), None) => {
                    return (Ok(((Some(a_ext),), a_out)), Some(OptState::None));
                }
                (Ok((a_ext, a_out)), Some(a_state)) => {
                    return (Ok(((Some(a_ext),), a_out)), Some(OptState::Some(a_state)));
                }
                (Err(e), None) => {
                    return (Err(e), Some(OptState::None));
                }
                (Err(e), Some(s)) => {
                    return (Err(e), Some(OptState::Some(s)));
                }
            },
            OptState::None => return (Ok(((None,), input)), None),
        }
    }

    fn regex(&self) -> String {
        format!("({})?", self.a.regex())
    }
}
