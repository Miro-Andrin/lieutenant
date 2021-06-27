use std::marker::PhantomData;

use crate::{
    generics::{Combine, CombinedTuples, Tuple},
    ParseError, IterParser,
};
pub struct And<A, B, World> {
    a: A,
    b: B,
    world: PhantomData<World>
}

impl<A,B,World> And<A,B,World> {
    pub fn new(a: A,b:B) -> Self {
        Self {
            a,
            b,
            world : PhantomData::default()
        }
    }
}

impl<A,B,World> PartialEq for And<A,B,World> where  A: PartialEq, B: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.a.eq(&other.a) && self.b.eq(&other.b)
    }
}

impl<A,B,World> Eq for And<A,B,World> where  A: Eq, B: Eq {}


#[derive(Debug, Clone)]
pub struct AndState<A, B> {
    a: A,
    b: B,
    look_for_a: bool,
}

impl<A, B> Default for AndState<A, B>
where
    A: Default,
    B: Default,
{
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            look_for_a: true,
        }
    }
}

impl<World, A, B> IterParser<World> for And<A, B,World>
where
    A: IterParser<World>,
    B: IterParser<World>,
    <<A as IterParser<World>>::Extract as Tuple>::HList:
        Combine<<<B as IterParser<World>>::Extract as Tuple>::HList>,
{
    type State = AndState<A::State, B::State>;
    type Extract = CombinedTuples<A::Extract, B::Extract>;

    fn iter_parse<'p>(
        &self,
        world: &World,
        mut state: <Self as IterParser<World>>::State,
        input: &'p str,
    ) -> (
        std::result::Result<(<Self as IterParser<World>>::Extract, &'p str), ParseError<'p>>,
        Option<<Self as IterParser<World>>::State>,
    ) {
        let mut best_error = None::<ParseError>;

        let (a_ext, a_out, more_a_states, a_state) = if state.look_for_a {
            loop {
                match self.a.iter_parse(world, state.a.clone(), input) {
                    (Ok((a_ext, a_out)), None) => {
                        // We found a match for a, its time we look for a match for b
                        state.look_for_a = false;
                        break (a_ext,a_out,false,None);
                    }
                    (Ok((a_ext, a_out)), Some(a_state)) => {
                        
                        // We found a match for a, its time we look for a match for b
                        state.look_for_a = false;
                        break (a_ext,a_out,true,Some(a_state));

                    },
                    (Err(e), None) => {
                        if e.got_further(&best_error) {
                            best_error = Some(e)
                        } 
                        return (Err(best_error.unwrap()), None);
                    }
                    (Err(e), Some(a_state)) => {    
                        if e.got_further(&best_error) {
                            best_error = Some(e)
                        }
                        state.a = a_state;
                        continue;
                    }
                }
            }
        } else {
            let (x, a_state) = self.a.iter_parse(world, state.a.clone(), input);
            let (a_ext, a_out) = x.unwrap();
            (a_ext, a_out,a_state.is_some(),a_state)
        };

        loop {
            match self.b.iter_parse(world, state.b, a_out) {
                (Ok((b_ext, b_out)), None) => {
                    // We found a match for b, but there are no more b states to consider
                    if !more_a_states {
                        // Then we have exhausted all states
                        return (Ok((a_ext.combine(b_ext), b_out)),None)
                    }
                    state.look_for_a = true;
                    state.b = Default::default();
                    state.a = a_state.unwrap();
                    return (Ok((a_ext.combine(b_ext), b_out)),None);
                }
                (Ok((b_ext, b_out)), Some(b_state)) => {
                    
                    // We found a match for a, its time we look for a match for b
                    state.look_for_a = false;
                    state.b = b_state;
                    return (Ok((a_ext.combine(b_ext),b_out)),Some(state));

                },
                (Err(e), None) => {
                    if e.got_further(&best_error) {
                        best_error = Some(e)
                    } 
                    
                    if !more_a_states {
                        return (Err(best_error.unwrap()), None)
                    }

                    state.look_for_a = true;
                    state.b = Default::default();
                    return (Err(best_error.unwrap()), None);
                }
                (Err(e), Some(b_state)) => {    
                    if e.got_further(&best_error) {
                        best_error = Some(e)
                    }
                    state.b = b_state;
                    continue;
                }
            }
        }


    }

    fn regex(&self) -> String {
        format!("({})({})",self.a.regex(), self.b.regex())
    }


}


// #[cfg(test)]
// mod test {
//     use crate::literal::Lit;

//     use super::And;


//     #[test]
//     fn test_and_literal() {
//         use crate::Parser;

//         let a = Lit {
//             str: "abc",
//         };

//         let b = Lit {
//             str: "def",
//         };

//         let and = And {
//             a,
//             b
//         };


//         let (res,_) = and.parse(&0,Default::default(),"abcdef");
//         assert!(res.is_ok());
//         let (res,out) = res.unwrap();
//         assert!(out.len() == 0);
//         assert!(res == ());
//     }
    
    
// }