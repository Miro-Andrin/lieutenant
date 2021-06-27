/*
    This module provides the ability to generate ranodm (ish)
    nfa, with a list of known examples that it should and should
    not recognise.
*/

use std::{collections::BTreeSet, iter};

mod regex;

use quickcheck::Arbitrary;

use super::NFA;
#[derive(Clone, Debug)]
enum RecipeNfa {
    Union {
        it: Box<RecipeNfa>,
        other: Box<RecipeNfa>,
    },

    FollowedBy {
        it: Box<RecipeNfa>,
        other: Box<RecipeNfa>,
    },

    Literal {
        lit: String,
    },

    Repeat {
        it: Box<RecipeNfa>,
    },

    NoOp {
        it: Box<RecipeNfa>,
    },
}

impl RecipeNfa {
    pub(crate) fn create(level: usize, gen: &mut quickcheck::Gen) -> Self {
        if level == 0 {
            let string = String::arbitrary(gen);
            return Self::Literal { lit: string };
        }

        if level < 4 {
            // Then give it a increased chance of returning a literal
            if 1 == *gen.choose(&[0, 0, 0, 1]).unwrap() {
                let string = String::arbitrary(gen);
                return Self::Literal { lit: string };
            }
        }

        match *gen.choose(&[0, 1, 2, 3]).unwrap() {
            0 => Self::Union {
                it: Box::new(Self::create(level - 1, gen)),
                other: Box::new(Self::create(level - 1, gen)),
            },

            1 => Self::FollowedBy {
                it: Box::new(Self::create(level - 1, gen)),
                other: Box::new(Self::create(level - 1, gen)),
            },

            2 => Self::Literal {
                lit: String::arbitrary(gen),
            },

            3 => Self::Repeat {
                it: Box::new(Self::create(level - 1, gen)),
            },

            _ => {
                unreachable!()
            }
        }
    }

    fn most_match(&self) -> BTreeSet<String> {
        match self {
            RecipeNfa::Union { it, other } => it
                .most_match()
                .union(&other.most_match())
                .cloned()
                .collect(),
            RecipeNfa::FollowedBy { it, other } => {
                let mut result = BTreeSet::<String>::new();

                for x in it.most_match() {
                    for y in other.most_match() {
                        result.insert(format!("{}{}", x, y));
                    }
                }

                result
            }
            RecipeNfa::Literal { lit } => iter::once(lit.clone()).collect(),
            RecipeNfa::Repeat { it } => {
                // We ofc cant list all cases, so we just add
                // the zero case, the regular case, and the double
                let mut result = it.most_match();

                // for x in it.most_match() {
                //     for y in it.most_match() {
                //         result.insert(format!("{}{}",x,y));
                //     }
                // }

                result.insert("".to_owned());
                result
            }
            RecipeNfa::NoOp { it } => it.most_match(),
        }
    }

    fn matches(&self, input: &str) -> BTreeSet<String> {
        match self {
            RecipeNfa::Union { it, other } => it
                .matches(input)
                .union(&other.matches(input))
                .cloned()
                .collect(),
            RecipeNfa::FollowedBy { it, other } => {
                let mut result = BTreeSet::<String>::new();

                for x in it.matches(input) {
                    result.extend(other.matches(x.as_str()));
                }

                result
            }
            RecipeNfa::Literal { lit } => {
                if input.starts_with(lit) {
                    let x = (&input[lit.len()..]).to_owned();
                    iter::once(x).collect()
                } else {
                    BTreeSet::new()
                }
            }
            RecipeNfa::Repeat { it } => {
                // We ofc cant list all cases, so we just add
                // the zero case, the regular case, and the double
                let mut result = it.matches(input);

                for x in result.clone() {
                    result.extend(it.matches(x.as_str()))
                }

                result.insert("".to_owned());
                result
            }
            RecipeNfa::NoOp { it: _ } => todo!(),
        }
    }

    pub fn nfa(&self) -> NFA {
        match &self {
            RecipeNfa::Union { it, other } => {
                let a = it.nfa();
                let b = other.nfa();
                return a.union(b).unwrap();
            }
            RecipeNfa::FollowedBy { it, other } => {
                let mut a = it.nfa();
                let b = other.nfa();
                a.followed_by(b).unwrap();
                return a;
            }
            RecipeNfa::Literal { lit } => {
                // Return a literal of length 1-4
                NFA::literal(lit).unwrap()
            }
            RecipeNfa::Repeat { it } => {
                let a = it.nfa();
                a.repeat().unwrap()
            }
            RecipeNfa::NoOp { it } => it.nfa(),
        }
    }

    // If the recipie contains a repeat, then matches ofcourse cant
    // returns every possible result.
    pub fn complete(&self) -> bool {
        match self {
            RecipeNfa::Union { it, other } => {
                if it.complete() {
                    return true;
                }
                other.complete()
            }
            RecipeNfa::FollowedBy { it, other } => {
                if it.complete() {
                    return true;
                }
                other.complete()
            }
            RecipeNfa::Literal { lit: _ } => true,
            RecipeNfa::Repeat { it: _ } => false,
            RecipeNfa::NoOp { it } => it.complete(),
        }
    }

}



impl quickcheck::Arbitrary for RecipeNfa {
    fn arbitrary(gen: &mut quickcheck::Gen) -> Self {
        Self::create(6, gen)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            RecipeNfa::Union { it, other } => {
                let mut vec = Vec::new();

                vec.push(Self::NoOp { it: it.clone() });

                vec.push(Self::NoOp { it: other.clone() });

                for it_shrinked in it.shrink() {
                    for other_shrinked in other.shrink() {
                        vec.push(Self::Union {
                            it: it_shrinked.clone(),
                            other: other_shrinked,
                        })
                    }
                }

                Box::new(vec.into_iter())
            }
            RecipeNfa::FollowedBy { it, other } => {
                let mut vec = Vec::new();

                vec.push(Self::NoOp { it: it.clone() });

                vec.push(Self::NoOp { it: other.clone() });

                for it_shrinked in it.shrink() {
                    for other_shrinked in other.shrink() {
                        vec.push(Self::FollowedBy {
                            it: it_shrinked.clone(),
                            other: other_shrinked,
                        })
                    }
                }

                Box::new(vec.into_iter())
            }
            RecipeNfa::Literal { lit } => {
                let mut vec = Vec::new();
                if lit.len() < 2 {
                    vec.push(Self::Literal { lit: lit.clone() })
                } else {
                    let chunks = lit
                        .chars()
                        .collect::<Vec<char>>()
                        .chunks(2)
                        .map(|c| c.iter().collect::<String>())
                        .collect::<Vec<String>>();

                    if let Some(fst) = chunks.first() {
                        vec.push(Self::Literal { lit: fst.clone() })
                    }

                    if let Some(lst) = chunks.last() {
                        vec.push(Self::Literal { lit: lst.clone() })
                    }
                }

                Box::new(vec.into_iter())
            }
            RecipeNfa::Repeat { it } => Box::new(it.shrink().map(|x| Self::Repeat { it: x })),
            RecipeNfa::NoOp { it } => Box::new(it.shrink().map(|x| Self::NoOp { it: x })),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[quickcheck]
    fn qc_random_nfa_1(recipie: RecipeNfa) -> bool {
        let nfa = recipie.nfa();
        let expected = recipie.most_match();
        for x in expected {
            if nfa.find(x).is_err() {
                return false;
            }
        }
        true
    }
}
