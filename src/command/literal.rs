use crate::{Dispatcher, AddToDispatcher, Node, NodeId, Validator};

use super::{Parser, Result};
use anyhow::bail;

use std::fmt;

/// Matches case ...
#[derive(Debug, Clone)]
pub struct Literal {
    value: String,
}

impl Parser for Literal {
    type Extract = ();

    #[inline]
    fn parse(&self, _input: &mut &str) -> Result<Self::Extract> {
        Ok(())
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
