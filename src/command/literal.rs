use crate::{AddToDispatcher, Dispatcher, Node, NodeId, Validator};

use super::{Parser, Result};
use anyhow::bail;

use std::fmt;

/// Matches case ...
#[derive(Debug, Clone)]
pub struct Literal {
    pub(crate) value: String,
}

impl Parser for Literal {
    type Extract = ();

    #[inline]
    fn parse(&self, input: &mut &str) -> Result<Self::Extract> {
        let mut value_lower = self.value.chars().flat_map(|c| c.to_lowercase());

        let x: Option<usize> = input
            .chars()
            .flat_map(|c| c.to_lowercase())
            .zip(&mut value_lower)
            .try_fold(0, |acc, (x, y)| {
                if x == y {
                    Some(acc + x.len_utf8())
                } else {
                    None
                }
            });

        if value_lower.next().is_some() {
            bail!("")
        }

        if let Some(end) = x {
            *input = &input[end..];
            Ok(())
        } else {
            bail!("")
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        Ok(())
    }
}

pub fn literal(literal: &str) -> Literal {
    Literal {
        value: literal.to_owned(),
    }
}

impl Validator for Literal {
    fn validate(&self, input: &mut &str) -> bool {
        self.value
            .chars()
            .flat_map(|c| c.to_lowercase())
            .zip(input.chars().flat_map(|c| c.to_lowercase()))
            .all(|(a, b)| a == b)
    }
}

impl AddToDispatcher for Literal {
    fn add_to_dispatcher(&self, parent: Option<NodeId>, dispatcher: &mut Dispatcher) -> NodeId {
        dispatcher.add(parent, Node::new(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = &mut "tp 10 10 10";
        assert!(lit.parse(input).is_ok());

        assert_eq!(input, &" 10 10 10");
    }

    #[test]
    fn empty() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = &mut "";
        assert!(lit.parse(input).is_err());
        assert_eq!(input, &"");
    }

    #[test]
    fn partial() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = &mut "temp";
        assert!(lit.parse(input).is_err());
        assert_eq!(input, &"temp");
    }

    #[test]
    fn longer_literal() {
        let lit = Literal {
            value: String::from("tp"),
        };

        let input = &mut "t";
        assert!(lit.parse(input).is_err());
        assert_eq!(input, &"t");
    }
}
