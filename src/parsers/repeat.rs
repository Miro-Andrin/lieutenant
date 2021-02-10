use std::todo;

use regex_syntax::hir::{self, Hir};

use super::{Parser, Result};

pub struct Repeat<P> {
    parser: P,
}

impl<P> Parser for Repeat<P>
where
    P: Parser,
{
    type Extract = (Vec<P::Extract>,);

    fn parse(&self, input: &mut &[&str]) -> Result<Self::Extract> {
        todo!()
    }

    fn regex(&self, syntax: &mut Vec<Hir>) {
        let mut hirs = Vec::new();
        self.parser.regex(&mut hirs);
        syntax.push(Hir::repetition(hir::Repetition {
            kind: hir::RepetitionKind::ZeroOrMore,
            greedy: true,
            hir: Box::new(Hir::group(hir::Group {
                kind: hir::GroupKind::NonCapturing,
                hir: Box::new(Hir::concat(hirs)),
            })),
        }));
    }
}

pub fn repeat<P>(parser: P) -> Repeat<P> {
    Repeat { parser }
}
