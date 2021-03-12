use super::parser::Parser;

pub struct Evaluator<'p, P> {
    parser: &'p P,
}

impl<'p, P: Parser> Evaluator<'p, P> {
    pub fn new(parser: &'p P) -> Self {
        Self { parser }
    }
}

impl<'p, P: Parser> Evaluator<'p, P> {
    pub fn evaluate<'i>(&self, input: &'i str) -> (Option<P::Extract>, &'i str) {
        let mut state = Some(P::ParserState::default());

        while let Some(st) = state {
            let (res, new_st) = self.parser.parse(st, input);

            match res {
                Ok((res, out)) => return (Some(res), out),
                Err(_) => {}
            }

            state = new_st;
        }

        (None, input)
    }

    pub fn evaluate_all<'i>(&self, input: &'i str) -> Vec<anyhow::Result<(P::Extract, &'i str)>> {
        let mut result = Vec::new();

        let mut state = Some(P::ParserState::default());

        while let Some(st) = state {
            let (res, new_st) = self.parser.parse(st, input);
            result.push(res);
            state = new_st;
        }

        return result;
    }
}
