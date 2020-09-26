use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Pattern<'a> {
    Literal(Cow<'a, str>),
    OneOf(Cow<'a, str>),
    Concat(&'a [Cow<'a, Self>]),
    Many(&'a Self),
    Alt(&'a [Cow<'a, Self>]),
    Optional(&'a Self),
    Not(&'a Self),
    OneOrMore(&'a Pattern<'a>),
}

impl<'a> Pattern<'a> {
    pub const SPACE: &'static Pattern<'static> = &literal(" ");
    pub const SPACE_MANY: &'static Pattern<'static> = &many(&Pattern::SPACE);
    pub const NOT_SPACE: &'static Pattern<'static> = &not(Pattern::SPACE);
    pub const WORD: &'static Pattern<'static> = &many(Pattern::NOT_SPACE);
    pub const DIGIT: &'static Pattern<'static> = &one_of("0123456789");
    pub const SPACE_MANY_ONE: &'static Pattern<'static> = &concat(&[
        Cow::Borrowed(Pattern::SPACE),
        Cow::Borrowed(Pattern::SPACE_MANY),
    ]);
    pub const SIGN: &'static Pattern<'static> = &one_of("-+");
    pub const NUMBER: &'static Pattern<'static> = &one_or_more(&Pattern::DIGIT);
    pub const NUMBER_OPTIONAL: &'static Pattern<'static> = &optional(Pattern::NUMBER);
    pub const SIGN_OPTIONAL: &'static Pattern<'static> = &optional(Pattern::SIGN);
    pub const INTEGER: &'static Pattern<'static> = &concat(&[
        Cow::Borrowed(Pattern::SIGN_OPTIONAL),
        Cow::Borrowed(Pattern::NUMBER),
    ]);
    pub const DOT: &'static Pattern<'static> = &literal(".");
    pub const FLOAT_ONE: &'static Pattern<'static> = &concat(&[
        Cow::Borrowed(Pattern::INTEGER),
        Cow::Borrowed(Pattern::DOT),
        Cow::Borrowed(Pattern::NUMBER_OPTIONAL),
    ]);
    pub const FLOAT_TWO: &'static Pattern<'static> = &concat(&[
        Cow::Borrowed(Pattern::SIGN_OPTIONAL),
        Cow::Borrowed(Pattern::DOT),
        Cow::Borrowed(Pattern::NUMBER),
    ]);
    // /// This pattern matches: [-.5; 5; -5; .5; 5.5]
    pub const FLOAT: &'static Pattern<'static> = &alt(&[
        Cow::Borrowed(Pattern::INTEGER),
        Cow::Borrowed(Pattern::FLOAT_ONE),
        Cow::Borrowed(Pattern::FLOAT_TWO),
    ]);
}

pub const fn literal<'a>(literal: &'a str) -> Pattern<'a> {
    Pattern::Literal(Cow::Borrowed(literal))
}

pub const fn one_of<'a>(one_of: &'a str) -> Pattern<'a> {
    Pattern::OneOf(Cow::Borrowed(one_of))
}

pub const fn concat<'a>(patterns: &'a [Cow<'a, Pattern<'a>>]) -> Pattern<'a> {
    Pattern::Concat(patterns)
}

pub const fn many<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
    Pattern::Many(pattern)
}

pub const fn alt<'a>(patterns: &'a [Cow<'a, Pattern<'a>>]) -> Pattern<'a> {
    Pattern::Alt(patterns)
}

pub const fn optional<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
    Pattern::Optional(pattern)
}

pub const fn not<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
    Pattern::Not(pattern)
}

pub const fn one_or_more<'a>(pattern: &'a Pattern<'a>) -> Pattern<'a> {
    Pattern::OneOrMore(pattern)
}

impl<'a> fmt::Display for Pattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Pattern::*;
        // match &self {
        //     Literal(lit) => write!(f, "\"{}\"", lit)?,
        //     OneOf(one_of) => write!(f, "[{}]", one_of)?,
        //     Concat(patterns) => {
        //         for pattern in patterns.iter() {
        //             write!(f, "{}", pattern)?
        //         }
        //     }
        //     Many(pattern) => write!(f, "{}*", pattern)?,
        //     Alt(patterns) => {
        //         write!(f, "(")?;
        //         for pattern in patterns.iter() {
        //             write!(f, "{}|", pattern)?;
        //         }
        //         write!(f, ")")?;
        //     }
        //     Optional(pattern) => write!(f, "({})?", pattern)?,
        // }
        Ok(())
    }
}
