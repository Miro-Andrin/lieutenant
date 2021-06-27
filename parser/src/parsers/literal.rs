use std::{marker::PhantomData};

use crate::{ParseError};


pub fn literal<World,S : AsRef<str>>(string: S) -> Lit<World> {
    Lit {
        a : string.as_ref().to_owned(),
        world : Default::default()
    }
}

pub struct Lit<World> {
    a: String,
    world: PhantomData<World>
}

impl<World> PartialEq for Lit<World> {
    fn eq(&self, other: &Self) -> bool {
        self.a.eq(&other.a)
    }
}

impl<World> Eq for Lit<World> {}

impl<World> Clone for Lit<World> {
    fn clone(&self) -> Self {
        Self {
            a : self.a.clone(),
            world : Default::default(),
        }
    }
}



fn overlap<'a>(it: &'a str, other: &str) -> (&'a str, &'a str) {
    it.split_at(other.len() - string_overlap_index(other, it))
}

fn string_overlap_index(left: &str, right: &str) -> usize {
    left.char_indices()
        .map(|(index, _)| index)
        .find(|index| {
            let slice_len = left.len() - index;
            slice_len <= right.len()
                && unsafe {
                    left.get_unchecked(*index..left.len())== right.get_unchecked(0..slice_len)
                }
        })
        .unwrap_or_else(|| left.len())
}



impl<World> crate::IterParser<World> for Lit<World> {
            
    type State = ();
    type Extract = ();
    
    fn iter_parse<'p>(
        &self,
        _world: &World,
        _state: Self::State,
        input: &'p str,
    ) -> (
        Result<(Self::Extract, &'p str), crate::ParseError<'p>>,
        Option<Self::State>,
    ) {

        let (overlap, rest) = overlap(input, self.a.as_str());

        if overlap.len() == self.a.len() {
            // Then we had a complete match
            return (Ok(((), rest)),None);
        }

        if overlap.is_empty() {
            return (Err(
                ParseError {
                    rest,
                    msg: format!("Expected the literal {}, but found no overlap.",self.a),
                }
            ),None);
        }   
        return (Err(
            ParseError {
                rest,
                msg: format!("Expected the literal {}, but found only the beginning: {}",self.a, overlap),
            }
        ),None);
    }

    fn regex(&self) -> String {
        regex_syntax::escape(&self.a)
    }
}

