use std::{fmt, marker::PhantomData, str::FromStr, todo};

use anyhow::{anyhow, bail};
use hir::{ClassUnicode, ClassUnicodeRange};
use regex_syntax::hir::{self, Hir};

use super::{Parser, Result, Syntax};

pub struct Argument<A> {
    pub(crate) argument: PhantomData<A>,
}

impl<A> Parser for Argument<A>
where
    A: FromStr + Syntax,
{
    type Extract = (A,);

    #[inline]
    fn parse(&self, input: &mut &[&str]) -> Result<Self::Extract> {
        let arg = A::from_str(&input[0]).map_err(|_| anyhow!("could not parse argument"))?;

        if input.is_empty() {
            bail!("input is empty!");
        }

        Ok((arg,))
    }

    #[inline]
    fn regex(&self, syntax: &mut Vec<Hir>) {
        let group_index = syntax.len();
        syntax.push(Hir::group(hir::Group {
            kind: hir::GroupKind::CaptureIndex(group_index as u32),
            hir: Box::new(A::regex()),
        }))
    }
}

pub fn argument<A>() -> Argument<A>
where
    A: FromStr,
{
    Argument {
        argument: Default::default(),
    }
}

impl<T> fmt::Display for Argument<T>
where
    T: Integer
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<int>")?;
        Ok(())
    }
}

impl<T> Syntax for T
where
    T: Integer,
{
    fn regex() -> Hir {
        let mut plus_minus = ClassUnicode::empty();
        plus_minus.push(ClassUnicodeRange::new('+', '+'));
        plus_minus.push(ClassUnicodeRange::new('-', '-'));
        
        let plus_minus = Hir::repetition(hir::Repetition {
            kind: hir::RepetitionKind::ZeroOrOne,
            greedy: true,
            hir: Box::new(Hir::class(hir::Class::Unicode(plus_minus))),
        });

        let mut digits = ClassUnicode::empty();
        digits.push(ClassUnicodeRange::new('0', '9'));
        let digits = Hir::repetition(hir::Repetition {
            kind: hir::RepetitionKind::OneOrMore,
            greedy: true,
            hir: Box::new(Hir::class(hir::Class::Unicode(digits))),
        });
        Hir::concat(vec![plus_minus, digits])
    }
}

pub trait Integer: FromStr {}

impl Integer for u8 {}
impl Integer for i8 {}
impl Integer for u16 {}
impl Integer for i16 {}
impl Integer for u32 {}
impl Integer for i32 {}
impl Integer for u64 {}
impl Integer for i64 {}
impl Integer for u128 {}
impl Integer for i128 {}
