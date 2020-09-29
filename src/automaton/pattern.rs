// use std::borrow::Cow;
// use std::fmt;

// #[derive(Debug, Eq, PartialEq, Clone)]
// pub enum Pattern<'a> {
//     Literal(Cow<'a, str>),
//     OneOf(Cow<'a, str>),
//     Concat(&'a [Cow<'a, Self>]),
//     Many(&'a Self),
//     Alt(&'a [Cow<'a, Self>]),
//     Optional(&'a Self),
//     Not(&'a Self),
//     OneOrMore(&'a Pattern<'a>),
// }


// pub const fn literal<'a>(literal: &'a str) -> Pattern<'a> {
//     Pattern::Literal(Cow::Borrowed(literal))
// }

// pub const fn one_of<'a>(one_of: &'a str) -> Pattern<'a> {
//     Pattern::OneOf(Cow::Borrowed(one_of))
// }

// pub const fn concat<'a>(patterns: &'a [Cow<'a, Pattern<'a>>]) -> Pattern<'a> {
//     Pattern::Concat(patterns)
// }

// pub const fn many<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
//     Pattern::Many(pattern)
// }

// pub const fn alt<'a>(patterns: &'a [Cow<'a, Pattern<'a>>]) -> Pattern<'a> {
//     Pattern::Alt(patterns)
// }

// pub const fn optional<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
//     Pattern::Optional(pattern)
// }

// pub const fn not<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
//     Pattern::Not(pattern)
// }

// pub const fn one_or_more<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
//     Pattern::OneOrMore(pattern)
// }

// impl<'a> fmt::Display for Pattern<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         use Pattern::*;
//         // match &self {
//         //     Literal(lit) => write!(f, "\"{}\"", lit)?,
//         //     OneOf(one_of) => write!(f, "[{}]", one_of)?,
//         //     Concat(patterns) => {
//         //         for pattern in patterns.iter() {
//         //             write!(f, "{}", pattern)?
//         //         }
//         //     }
//         //     Many(pattern) => write!(f, "{}*", pattern)?,
//         //     Alt(patterns) => {
//         //         write!(f, "(")?;
//         //         for pattern in patterns.iter() {
//         //             write!(f, "{}|", pattern)?;
//         //         }
//         //         write!(f, ")")?;
//         //     }
//         //     Optional(pattern) => write!(f, "({})?", pattern)?,
//         // }
//         Ok(())
//     }
// }
