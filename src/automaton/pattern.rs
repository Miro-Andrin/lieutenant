use std::borrow::Cow;
use std::fmt;

#[macro_export]
macro_rules! pattern {
    // Concat
    (concat; $body:tt;) => ($body);
    ([$($body:tt)*]; $lit:literal* $($remaining:tt)*) => {
        pattern!(concat; [$($body)* pattern!($lit*),]; $($remaining)*);
    };
    (concat; [$($body:tt)*]; $lit:literal $($remaining:tt)*) => {
        pattern!(concat; [$($body)* pattern!($lit),]; $($remaining)*);
    };
    (concat; [$($body:tt)*]; [$one_of:literal]* $($remaining:tt)*) => {
        pattern!(concat; [$($body)* pattern!([$one_of]*),]; $($remaining)*);
    };
    (concat; [$($body:tt)*]; [$one_of:literal] $($remaining:tt)*) => {
        pattern!(concat; [$($body)* pattern!([$one_of]),]; $($remaining)*);
    };
    // Patterns
    ($lit:literal*) => (Pattern::Many(&pattern!($lit)));
    ($lit:literal) => {{
        Pattern::Literal(::std::borrow::Cow::Borrowed($lit))
    }};
    ([$one_of:literal]*) => (Pattern::Many(&pattern!([$one_of])));
    ([$one_of:literal]) => {{
        Pattern::OneOf(::std::borrow::Cow::Borrowed($one_of))
    }};
    ($($all:tt)*) => {{
        Pattern::Concat(&pattern!(concat; []; $($all)*))
    }};
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Pattern<'a> {
    Literal(Cow<'a, str>),
    OneOf(Cow<'a, str>),
    Concat(&'a [Self]),
    Many(&'a Self),
    Alt(&'a [Self]),
    Optional(&'a Self),
}

impl<'a> Pattern<'a> {
    pub const WORD: Pattern<'static> = pattern!([""]*);
    pub const DIGIT: Pattern<'static> = pattern!(["0123456789"]);
    pub const SPACE: Pattern<'static> = pattern!(" ");
    pub const SPACE_MANY_ONE: Pattern<'static> = Pattern::concat(&[pattern!(" "), pattern!(" "*)]);

    pub fn literal(literal: &'a str) -> Self {
        Self::Literal(Cow::from(literal))
    }

    pub fn one_of(one_of: &'static str) -> Self {
        Self::OneOf(Cow::from(one_of))
    }

    pub const fn concat(patterns: &'a [Self]) -> Self {
        Self::Concat(patterns)
    }

    pub const fn many(pattern: &'a Self) -> Self {
        Self::Many(pattern)
    }

    pub const fn alt(patterns: &'a [Self]) -> Self {
        Self::Alt(patterns)
    }

    pub const fn optional(pattern: &'a Self) -> Self {
        Self::Optional(pattern)
    }
}

impl<'a> fmt::Display for Pattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Pattern::*;
        match &self {
            Literal(lit) => write!(f, "\"{}\"", lit)?,
            OneOf(one_of) => write!(f, "[{}]", one_of)?,
            Concat(patterns) => {
                for pattern in patterns.iter() {
                    write!(f, "{}", pattern)?
                }
            }
            Many(pattern) => write!(f, "{}*", pattern)?,
            Alt(patterns) => {
                write!(f, "(")?;
                for pattern in patterns.iter() {
                    write!(f, "{}|", pattern)?;
                }
                write!(f, ")")?;
            },
            Optional(pattern) => write!(f, "({})?", pattern)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let pattern = pattern!("0123456789");
        assert_eq!(pattern, Pattern::Literal("0123456789".into()));

        let pattern = pattern!(["0123456789"]);
        assert_eq!(pattern, Pattern::OneOf("0123456789".into()));

        let pattern = pattern!("0123456789"*);
        assert_eq!(pattern, Pattern::Many(&Pattern::literal("0123456789")));

        let pattern = pattern!(["0123456789"]*);
        assert_eq!(pattern, Pattern::Many(&Pattern::OneOf("0123456789".into())));

        let pattern = pattern!("abc""123");
        assert_eq!(
            pattern,
            Pattern::Concat(&[
                Pattern::Literal("abc".into()),
                Pattern::Literal("123".into())
            ])
        );

        let pattern = pattern!(["0123456789"]["0123456789"]*);
        assert_eq!(
            pattern,
            Pattern::Concat(&[
                Pattern::OneOf("0123456789".into()),
                Pattern::Many(&Pattern::one_of("0123456789"))
            ])
        );
    }
}
