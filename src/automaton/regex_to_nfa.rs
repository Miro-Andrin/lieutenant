use std::ops::Range;

use regex_syntax::{hir::ClassUnicodeRange, Parser};

use super::{ByteClass, StateId, NFA};
use anyhow::{bail, Result};

pub(crate) fn regex_to_nfa(regex: &str) -> Result<NFA> {
    let hir = Parser::new().parse(regex)?;
    hir_to_nfa(&hir)
}

impl From<&ClassUnicodeRange> for NFA {
    fn from(range: &ClassUnicodeRange) -> Self {

        
        let mut start: [u8;4] = [0;4];
        range.start().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        range.end().encode_utf8(&mut end);

        let start_len = range.start().len_utf8();
        let end_len = range.end().len_utf8();



        match (start_len,end_len) {
            (1,1) => {
                println!("(1,1) CASE");
                //Ascii
                let mut nfa = NFA::empty();
                nfa.end = nfa.push_state();

                let mut transition = [0u8;256];
                for i in start[0]..=end[0] {
                    transition[i as usize] = 1;
                }
                nfa.set_transitions(nfa.start, ByteClass(transition.to_vec()), vec![vec![],vec![nfa.end]]);

                println!("nfa internal: {:?}",nfa);
                return nfa;
            },
            (1,2) => {
                println!("(1,2) CASE");
                //match any ascii bigger then or equal to start
                let mut ascii = NFA::empty();
                ascii.end = ascii.push_state();

                let mut transition = [0u8;256];
                for i in start[0]..128 {
                    transition[i as usize] = 1;
                }
                ascii.set_transitions(ascii.start, ByteClass(transition.to_vec()), vec![vec![],vec![ascii.end]]);
                                
                //      (2)_
                //     /    \   
                // start     end
                //    \     /
                //     (1)-
                let mut byte2 = NFA::empty();
                let states = [byte2.push_state(),byte2.push_state()];
                byte2.end = byte2.push_state();


                //Set transition from start
                let mut transition = [0u8;256];
                for i in 192..end[0] {
                    transition[i as usize] = 1;
                }
                transition[end[0] as usize] = 2;
                byte2.set_transitions(byte2.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[0]],vec![states[1]]]);
                

                //Set transition from one to be anything starting with 10xx_xxxx
                transition = [0u8;256];
                for i in 120..192{
                    transition[i as usize] = 1;
                }            
                byte2.set_transitions(states[0], ByteClass::full(), vec![vec![],vec![byte2.end]]);


                //Set transition from 2 to be any values less then or equal to end[1]
                transition = [0u8;256];
                for i in 128..=end[1]{
                    transition[i as usize] = 1;
                }
                byte2.set_transitions(states[1], ByteClass(transition.to_vec()), vec![vec![],vec![byte2.end]]);
                
                println!("start:{:?} end:{:?}, byte2: {:?}",start,end,byte2);


                return ascii.union(&byte2);


            },
            (1,3) => {
                println!("(1,3) CASE");

                //match any ascii bigger then or equal to start
                let mut ascii = NFA::empty();
                ascii.end = ascii.push_state();

                let mut transition = [0u8;256];
                for i in start[0]..128 {
                    transition[i as usize] = 1;
                }
                ascii.set_transitions(ascii.start, ByteClass(transition.to_vec()), vec![vec![],vec![ascii.end]]);
                         

                //Match any two byte utf-8 
                //
                // start -> 1 -> end
                //
                let mut byte2 = NFA::empty();
                let states = [byte2.push_state()];
                byte2.end = byte2.push_state();

                //start -> 1
                let mut transition = [0u8;256];
                for i in 192..224 {
                    transition[i] = 1;
                }
                byte2.set_transitions(byte2.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[0]]]);

                // 1 -> end
                transition = [0u8;256];
                for i in 128..192 {
                    transition[i] = 1;
                }
                byte2.set_transitions(states[0], ByteClass(transition.to_vec()), vec![vec![],vec![byte2.end]]);



                //Match any 3 byte utf-8 less then end
                //
                //         (2) -- (3)     
                //       /    \      \
                // start       \      end
                //       \      \   /            
                //        (1) - (4)
                

                // start -> 2    (eq end[0])                              x
                // start -> 1    (lt end[0] and bigger then or equal 224) x
                // 1     -> 4    any in 128..192                          x
                // 2     -> 4    lt end[1]                                x
                // 4     -> end  any in 128..192                          x
                // 2     -> 3    (eq end[1])                              x
                // 3     -> end  (eq or lt end[2])                        x

                let mut byte3 = NFA::empty();
                let states = [byte3.push_state(),byte3.push_state(),byte3.push_state(),byte3.push_state()];
                byte3.end = byte3.push_state();

                //Start -> 1 (lt end[0])
                //Start -> 2 (eq end[0])
                let mut transition = [0u8;256];
                transition[end[0] as usize] = 1;
                 for i in 224..end[0] {
                    transition[i as usize] = 2;
                }
                byte3.set_transitions(byte3.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[1]],vec![states[0]]]);


                //1 -> 4  and 4 -> end  (any valid 2d or 3df byte utf-8, so 10xxxxxx)
                transition = [0u8;256];
                for i in 128..192 {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(states[0], ByteClass(transition.to_vec()) , vec![vec![],vec![states[3]]]);
                byte3.set_transitions(states[3], ByteClass(transition.to_vec()) , vec![vec![],vec![byte3.end]]);

                
                //2 -> 3 (eq end[1])
                transition = [0u8;256];
                transition[end[1] as usize] = 1;  // 2 -> 3  eq end[1]
                for i in 128..end[1] {
                    transition[i as usize] = 2;
                }
                byte3.set_transitions(states[1], ByteClass(transition.to_vec()) , vec![vec![],vec![states[2]],vec![states[3]]]);

                // 3 -> end  (10xxxxxx byte less then end[2])
                transition = [0u8;256];
                for i in 128..=end[2] {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(states[2], ByteClass(transition.to_vec()) , vec![vec![],vec![byte3.end]]);


                return byte3.union(&byte2.union(&ascii))

            },
            (1,4) => {

                println!("(1,4) CASE");

                //match any ascii bigger then or equal to start
                let mut ascii = NFA::empty();
                ascii.end = ascii.push_state();

                let mut transition = [0u8;256];
                for i in start[0]..128 {
                    transition[i as usize] = 1;
                }
                ascii.set_transitions(ascii.start, ByteClass(transition.to_vec()), vec![vec![],vec![ascii.end]]);
                         

                //Match any two byte utf-8 
                //
                // start -> 1 -> end
                //
                let mut byte2 = NFA::empty();
                let states = [byte2.push_state()];
                byte2.end = byte2.push_state();

                //start -> 1
                let mut transition = [0u8;256];
                for i in 192..224 {
                    transition[i] = 1;
                }
                byte2.set_transitions(byte2.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[0]]]);

                // 1 -> end
                transition = [0u8;256];
                for i in 128..192 {
                    transition[i] = 1;
                }
                byte2.set_transitions(states[0], ByteClass(transition.to_vec()), vec![vec![],vec![byte2.end]]);


                // Match any 3 byte utf-8
                //
                // start -> 1 -> 2 -> end                
                //
                let mut byte3 = NFA::empty();
                let states = [byte3.push_state(),byte3.push_state()];
                byte3.end = byte3.push_state();

                // start -> 1  11100000 -> 11101111   224..240
                let mut transition = [0u8;256];
                for i in 224..240 {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(byte3.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[0]]]);
                
                let mut transition = [0u8;256];
                for i in 128..192 {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(states[0], ByteClass(transition.to_vec()), vec![vec![],vec![states[1]]]);
                byte3.set_transitions(states[1], ByteClass(transition.to_vec()), vec![vec![],vec![byte3.end]]);




                // Match any four byte utf-8 less then end
                // 
                //         (2) --- (3) --- (6)     
                //       /    \      \       \
                // start       \      \       end
                //       \      \      \    /       
                //        (1) - (4) -- (5)-                    
                // 

                
                // start -> 2    (eq end[0])                               x
                // start -> 1    (lt end[0] and bigger then or equal 224)  x
                // 1     -> 4    any in 128..192                          
                // 2     -> 4    lt end[1]                                
                // 2     -> 3    (eq end[1])                                                  
                // 3     -> 6    (eq end[2])
                // 6     -> end  (lt eq end[3])
                // 5     -> end  any in 128..192 
                // 3     -> 5    (lt end[2])
                // 4     -> 5    any in 128..192 

                let mut byte4 = NFA::empty();
                let states = [byte4.push_state(),byte4.push_state(),byte4.push_state(),byte4.push_state(),byte4.push_state(),byte4.push_state()];
                byte4.end = byte3.push_state();


                // start -> 2    (eq end[0])                               x
                // start -> 1    (lt end[0] and bigger then or equal 224)  x
                let mut transition = [0u8;256];
                transition[end[0] as usize] = 1; // eq end[0] -> states[1]
                for n in 240..end[0] {
                    transition[n as usize] = 2; // lt end[0] -> states[0]
                }
                byte4.set_transitions(byte4.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[1]],vec![states[0]]]);

            
                // 2     -> 3    eq end[1]
                // 2     -> 4    lt end[1]
                let mut transition = [0u8;256];
                transition[end[1] as usize] = 1;
                for i in 128..end[1]{
                    transition[i as usize] = 2;
                }
                byte4.set_transitions(states[1], ByteClass(transition.to_vec()), vec![vec![],vec![states[2]],vec![states[3]]]);
                
                // 3     -> 6    (eq end[2])
                // 3     -> 5    (lt end[2])
                let mut transition = [0u8;256];
                transition[end[2] as usize] = 1;
                for i in 128..end[2]{
                    transition[i as usize] = 2;
                }
                byte4.set_transitions(states[2], ByteClass(transition.to_vec()), vec![vec![],vec![states[5]],vec![states[4]]]);
                
                
                // 6     -> end  (lt eq end[3])
                let mut transition = [0u8;256];
                for i in 128..=end[3]{
                    transition[i as usize] = 1;
                }
                byte4.set_transitions(states[5], ByteClass(transition.to_vec()), vec![vec![],vec![byte4.end]]);
                

                // 4     -> 5    any in 128..192 
                // 1     -> 4    any in 128..192
                // 5     -> end  any in 128..192
                let mut transition = [0u8;256];
                for i in 128..=192{
                    transition[i as usize] = 1;
                }
                byte4.set_transitions(states[3], ByteClass(transition.to_vec()), vec![vec![],vec![states[4]]]);
                byte4.set_transitions(states[0], ByteClass(transition.to_vec()), vec![vec![],vec![states[3]]]);
                byte4.set_transitions(states[4], ByteClass(transition.to_vec()), vec![vec![],vec![byte4.end]]);
                
                return byte4.union(&ascii.union(&byte3.union(&byte2)));


            },
            (2,1) => {
                unreachable!()
            },
            (2,2) => {

                println!("(2,2) CASE");
                
                
                //          -1-
                //        /     \     
                //   start---2--end
                //        \     /
                //          -3-

                // start -> 1  (eq end[0])
                // start -> 2  (start[0]+1..states[1])
                // start -> 3  (eq start[0])
                // 1 -> end    (128..=end[1]) lt eq end[1] but valid 2nd byte in utf8
                // 2 -> end    any  128..192   10xx xxxx
                // 3 -> end    start[1]..192  gt eq start[1] but valid 2nd byte in utf8


                // start -> 1  (eq end[0])
                // start -> 2  (start[0]+1..states[1])
                // start -> 3  (eq start[0])
                let mut byte2 = NFA::empty();
                let states: [StateId; 3] = [byte2.push_state(),byte2.push_state(),byte2.push_state()];
                byte2.end = byte2.push_state();                    

                let mut transitions = [0u8;256];
                transitions[end[0] as usize] = 1;
                for i in start[0]+1..end[0] {                        
                    transitions[i as usize] = 2;
                }
                transitions[end[0] as usize] = 3;
                byte2.set_transitions(byte2.start, ByteClass(transitions.to_vec()), vec![vec![],vec![states[0]],vec![states[1]],vec![states[2]]]);

                // 1 -> end    (128..=end[1]) lt eq end[1] but valid 2nd byte in utf8
                let mut transitions = [0u8;256];
                for i in 128..end[1] {
                    transitions[i as usize] = 1;
                }
                byte2.set_transitions(states[0], ByteClass(transitions.to_vec()), vec![vec![],vec![byte2.end]]);
                
                // 2 -> end    any  128..192   10xx xxxx
                let mut transitions = [0u8;256];
                for i in 128..192 {
                    transitions[i as usize] = 1;
                }
                byte2.set_transitions(states[1], ByteClass(transitions.to_vec()), vec![vec![],vec![byte2.end]]);
                
                // 3 -> end    start[1]..192  gt eq start[1] but valid 2nd byte in utf8
                let mut transitions = [0u8;256];
                for i in start[1]..192 {
                    transitions[i as usize] = 1;
                }
                byte2.set_transitions(states[2], ByteClass(transitions.to_vec()), vec![vec![],vec![byte2.end]]);
                

                return byte2

            },
            (2,3) => {

                //          - 1 -
                //        /       \
                //   start         end
                //        \       /
                //          - 2 -

                // start -> 1   eq start[0] 
                // start -> 2   gt start[0]  so start[0]+1..192
                //     2 -> end 128..192
                //     1 -> end gt or eq satart[9] and less then 192
                
                let mut byte2 = NFA::empty();
                let states: [StateId; 2] = [byte2.push_state(),byte2.push_state()];
                byte2.end = byte2.push_state();
                
                // start -> 1   eq start[0] 
                // start -> 2   gt start[0]  so start[0]+1..192
                let mut transition = [0u8;256];
                transition[start[0] as usize] = 1;
                for i in start[0] +1 ..192 {
                    transition[i as usize] = 2;
                }
                byte2.set_transitions(byte2.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[0]],vec![states[1]]]);



                //Match any 3 byte utf-8 less then end
                //
                //         (2) -- (3)     
                //       /    \      \
                // start       \      end
                //       \      \   /            
                //        (1) - (4)
                

                // start -> 2    (eq end[0])                              x
                // start -> 1    (lt end[0] and bigger then or equal 224) x
                // 1     -> 4    any in 128..192                          x
                // 2     -> 4    lt end[1]                                x
                // 4     -> end  any in 128..192                          x
                // 2     -> 3    (eq end[1])                              x
                // 3     -> end  (eq or lt end[2])                        x

                let mut byte3 = NFA::empty();
                let states = [byte3.push_state(),byte3.push_state(),byte3.push_state(),byte3.push_state()];
                byte3.end = byte3.push_state();

                //Start -> 1 (lt end[0])
                //Start -> 2 (eq end[0])
                let mut transition = [0u8;256];
                transition[end[0] as usize] = 1;
                 for i in 224..end[0] {
                    transition[i as usize] = 2;
                }
                byte3.set_transitions(byte3.start, ByteClass(transition.to_vec()), vec![vec![],vec![states[1]],vec![states[0]]]);


                //1 -> 4  and 4 -> end  (any valid 2d or 3df byte utf-8, so 10xxxxxx)
                transition = [0u8;256];
                for i in 128..192 {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(states[0], ByteClass(transition.to_vec()) , vec![vec![],vec![states[3]]]);
                byte3.set_transitions(states[3], ByteClass(transition.to_vec()) , vec![vec![],vec![byte3.end]]);

                
                //2 -> 3 (eq end[1])
                transition = [0u8;256];
                transition[end[1] as usize] = 1;  // 2 -> 3  eq end[1]
                for i in 128..end[1] {
                    transition[i as usize] = 2;
                }
                byte3.set_transitions(states[1], ByteClass(transition.to_vec()) , vec![vec![],vec![states[2]],vec![states[3]]]);

                // 3 -> end  (10xxxxxx byte less then end[2])
                transition = [0u8;256];
                for i in 128..=end[2] {
                    transition[i as usize] = 1;
                }
                byte3.set_transitions(states[2], ByteClass(transition.to_vec()) , vec![vec![],vec![byte3.end]]);


                byte2.union(&byte3)

            },
            (2,4) => {
                
                
                


                todo!()
            },
            (3,1) => {
                todo!()
            },
            (3,2) => {
                todo!()
            },
            (3,3) => {
                todo!()
            },
            (4,4) => {
                todo!()
            },

            _ => unreachable!()
            
        }





      


        


    }
}

fn hir_to_nfa(hir: &regex_syntax::hir::Hir) -> Result<NFA> {
    match hir.kind() {
        regex_syntax::hir::HirKind::Empty => Ok(NFA::single_u8()),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(uni) => Ok(NFA::literal(&uni.to_string())),
            regex_syntax::hir::Literal::Byte(byte) => Ok(NFA::literal(&byte.to_string())),
        },
        regex_syntax::hir::HirKind::Class(class) => {
            match class {
                regex_syntax::hir::Class::Unicode(uni) => {
                    
                    let mut nfa = NFA::empty();
                    for range in uni.ranges() {
                        nfa = nfa.union(&NFA::from(range));
                    }

                    Ok(nfa)
                }
                regex_syntax::hir::Class::Bytes(byte) => {
                    let mut nfa = NFA::empty();
                    for range in byte.iter() {
                        //Todo check that range is inclusive
                        nfa = nfa.union(&NFA::from(Range {
                            start: range.start(),
                            end: range.end(),
                        }));
                    }
                    Ok(nfa)
                }
            }
        }
        regex_syntax::hir::HirKind::Anchor(x) => match x {
            regex_syntax::hir::Anchor::StartLine => bail!("We dont suport StartLine symbols!"),
            regex_syntax::hir::Anchor::EndLine => bail!("We dont suport EndLine symbols!"),
            regex_syntax::hir::Anchor::StartText => bail!("We dont suport StartText symbol!"),
            regex_syntax::hir::Anchor::EndText => bail!("We dont suport EndText symbol!"),
        },
        regex_syntax::hir::HirKind::WordBoundary(boundary) => {
            match boundary {
                regex_syntax::hir::WordBoundary::Unicode => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::UnicodeNegate => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::Ascii => {
                    todo!() // I dont know if we need to suport this
                }
                regex_syntax::hir::WordBoundary::AsciiNegate => {
                    todo!() // I dont know if we need to suport this
                }
            }
        }
        regex_syntax::hir::HirKind::Repetition(x) => {
            if x.greedy {
                let nfa = hir_to_nfa(&x.hir)?;
                Ok(nfa.repeat())
            } else {
                bail!("We dont suport non greedy patterns")
            }
        }
        regex_syntax::hir::HirKind::Group(group) => {
            //TODO i dont know how we are suposed to interprite an empty
            //hir/nfa in this case. Should it maybe be a no-op?
            hir_to_nfa(&group.hir)
        }
        regex_syntax::hir::HirKind::Concat(cats) => {
            let mut nfas = cats.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst = fst.concat(&nfa?);
            }
            Ok(fst)
        }

        regex_syntax::hir::HirKind::Alternation(alts) => {
            let mut nfas = alts.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst = fst.union(&nfa?);
            }
            Ok(fst)
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::automaton::{DFA, Find};
    use super::*;
    #[test]
    fn abc() {
        let nfa = regex_to_nfa("abc").unwrap();
        let dfa = DFA::from(nfa);
        assert!(dfa.find("").is_err());
        assert!(dfa.find("abc").is_ok());
    }

    #[test]
    fn simple() {
        let nfa = regex_to_nfa("[a]").unwrap();
        println!("{:?}",nfa);
        assert!(nfa.start != nfa.end);
        
        let dfa = DFA::from(nfa);
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("a").is_ok());
        assert!(dfa.find("").is_err());
    }


    #[test]
    fn range11() {
        let nfa = regex_to_nfa("[a-z]").unwrap();
        let dfa = DFA::from(nfa);
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("a").is_ok());
        assert!(dfa.find("").is_err());
    }
    
    #[test]
    fn range12() {
        let from = "r";
        let to = "¬";
        let nfa = regex_to_nfa("[r-¬]").unwrap();
        let dfa = DFA::from(nfa);
        let mut temp = [0u8;4];


        let mut start: [u8;4] = [0;4];
        from.chars().nth(0).unwrap().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        to.chars().nth(0).unwrap().encode_utf8(&mut end);

    
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("r").is_ok());
        assert!(dfa.find("s").is_ok());
        assert!(dfa.find("~").is_ok());
        //'ƒ'.encode_utf8(&mut temp);
        //println!("ƒ: {:?}",temp);
        assert!(dfa.find("|").is_ok());
        assert!(dfa.find("ƒ").is_err()); // start:[114, 0, 0, 0] end:[194, 172, 0, 0]  ƒ: [198, 146, 0, 0]
        

        assert!(dfa.find("ƒ").is_err());
        assert!(dfa.find("®").is_err());
        assert!(dfa.find("«").is_ok());
        assert!(dfa.find("ª").is_ok());
        assert!(dfa.find("z").is_ok());

    }


    #[test]
    fn range13() {
        let from = "a";
        let to = "ॐ";
        let nfa = regex_to_nfa("[a-ॐ]").unwrap();
        let dfa = DFA::from(nfa);
        let mut temp = [0u8;4];


        let mut start: [u8;4] = [0;4];
        from.chars().nth(0).unwrap().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        to.chars().nth(0).unwrap().encode_utf8(&mut end);

    
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("r").is_ok());
        assert!(dfa.find("s").is_ok());
        assert!(dfa.find("~").is_ok());
        //'ƒ'.encode_utf8(&mut temp);
        //println!("ƒ: {:?}",temp);
        assert!(dfa.find("|").is_ok());
        assert!(dfa.find("ƒ").is_ok()); // start:[114, 0, 0, 0] end:[194, 172, 0, 0]  ƒ: [198, 146, 0, 0]
        

        assert!(dfa.find("ƒ").is_ok());
        assert!(dfa.find("®").is_ok());
        assert!(dfa.find("«").is_ok());
        assert!(dfa.find("ª").is_ok());
        assert!(dfa.find("z").is_ok());
    


        // 'ॉ'.encode_utf8(&mut temp);
        // println!("ॉ: {:?}",temp);

        // 'ॐ'.encode_utf8(&mut temp);
        // println!("ॐ: {:?}",temp);

        // 'ख़'.encode_utf8(&mut temp);
        // println!("ख़: {:?}",temp);

        assert!(dfa.find("ॉ").is_ok());
        assert!(dfa.find("ऽ").is_ok());
        assert!(dfa.find("ऴ").is_ok());
        assert!(dfa.find("ॐ").is_ok());
        assert!(dfa.find("ख़").is_err());
    }

    #[test]
    fn range14() {
        let from = "a";
        let to = "🤡";
        let nfa = regex_to_nfa("[a-🤡]").unwrap();
        let dfa = DFA::from(nfa);
        let mut temp = [0u8;4];


        let mut start: [u8;4] = [0;4];
        from.chars().nth(0).unwrap().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        to.chars().nth(0).unwrap().encode_utf8(&mut end);

    
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("r").is_ok());
        assert!(dfa.find("s").is_ok());
        assert!(dfa.find("~").is_ok());
        //'ƒ'.encode_utf8(&mut temp);
        //println!("ƒ: {:?}",temp);
        assert!(dfa.find("|").is_ok());
        assert!(dfa.find("ƒ").is_ok()); // start:[114, 0, 0, 0] end:[194, 172, 0, 0]  ƒ: [198, 146, 0, 0]
        

        assert!(dfa.find("ƒ").is_ok());
        assert!(dfa.find("®").is_ok());
        assert!(dfa.find("«").is_ok());
        assert!(dfa.find("ª").is_ok());
        assert!(dfa.find("z").is_ok());
    


        // 'ॉ'.encode_utf8(&mut temp);
        // println!("ॉ: {:?}",temp);

        // 'ॐ'.encode_utf8(&mut temp);
        // println!("ॐ: {:?}",temp);

        // 'ख़'.encode_utf8(&mut temp);
        // println!("ख़: {:?}",temp);

        assert!(dfa.find("ॉ").is_ok());
        assert!(dfa.find("ऽ").is_ok());
        assert!(dfa.find("ऴ").is_ok());
        assert!(dfa.find("ॐ").is_ok());
        assert!(dfa.find("ख़").is_ok());

        assert!(dfa.find("🤖").is_ok());
        assert!(dfa.find("🟣").is_ok());
        assert!(dfa.find("🗨").is_ok());
        assert!(dfa.find("👵").is_ok());
        assert!(dfa.find("🎃").is_ok());
        assert!(dfa.find("🌶").is_ok());
        assert!(dfa.find("🌐").is_ok());
        assert!(dfa.find("䒗").is_ok());
        assert!(dfa.find("").is_ok());
        assert!(dfa.find("ϲ").is_ok());
        assert!(dfa.find("ϫ").is_ok());
        assert!(dfa.find("ζ").is_ok());
        assert!(dfa.find("Θ").is_ok());


        assert!(dfa.find("#").is_err());
        assert!(dfa.find("1").is_err());
        assert!(dfa.find("8").is_err());
        assert!(dfa.find("`").is_err());
        assert!(dfa.find("🤢").is_err()); //One after clown
        
    }


    #[test]
    fn range22() {
        let from = "Ð";
        let to = "Ù";
        let nfa = regex_to_nfa("[Ð-Ù]").unwrap();
        let dfa = DFA::from(nfa);
        let mut temp = [0u8;4];


        let mut start: [u8;4] = [0;4];
        from.chars().nth(0).unwrap().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        to.chars().nth(0).unwrap().encode_utf8(&mut end);

        
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("r").is_err());
        assert!(dfa.find("s").is_err());
        assert!(dfa.find("~").is_err());
        //'ƒ'.encode_utf8(&mut temp);
        //println!("ƒ: {:?}",temp);
        assert!(dfa.find("|").is_err());
        assert!(dfa.find("ƒ").is_err()); // start:[114, 0, 0, 0] end:[194, 172, 0, 0]  ƒ: [198, 146, 0, 0]
        

        assert!(dfa.find("ƒ").is_err());
        assert!(dfa.find("®").is_err());
        assert!(dfa.find("«").is_err());
        assert!(dfa.find("ª").is_err());
        assert!(dfa.find("z").is_err());
    


        // 'ॉ'.encode_utf8(&mut temp);
        // println!("ॉ: {:?}",temp);

        // 'ॐ'.encode_utf8(&mut temp);
        // println!("ॐ: {:?}",temp);

        // 'ख़'.encode_utf8(&mut temp);
        // println!("ख़: {:?}",temp);

        assert!(dfa.find("ॉ").is_err());
        assert!(dfa.find("ऽ").is_err());
        assert!(dfa.find("ऴ").is_err());
        assert!(dfa.find("ॐ").is_err());
        assert!(dfa.find("ख़").is_err());

        assert!(dfa.find("🤖").is_err());
        assert!(dfa.find("🟣").is_err());
        assert!(dfa.find("🗨").is_err());
        assert!(dfa.find("👵").is_err());
        assert!(dfa.find("🎃").is_err());
        assert!(dfa.find("🌶").is_err());
        assert!(dfa.find("🌐").is_err());
        assert!(dfa.find("䒗").is_err());
        assert!(dfa.find("").is_err());
        assert!(dfa.find("ϲ").is_err());
        assert!(dfa.find("ϫ").is_err());
        assert!(dfa.find("ζ").is_err());
        assert!(dfa.find("Θ").is_err());


        assert!(dfa.find("#").is_err());
        assert!(dfa.find("1").is_err());
        assert!(dfa.find("8").is_err());
        assert!(dfa.find("`").is_err());
        assert!(dfa.find("🤢").is_err()); //One after clown
        
    }

    #[test]
    fn range23() {
        let from = "a";
        let to = "🤡";
        let nfa = regex_to_nfa("[a-🤡]").unwrap();
        let dfa = DFA::from(nfa);
        let mut temp = [0u8;4];


        let mut start: [u8;4] = [0;4];
        from.chars().nth(0).unwrap().encode_utf8(&mut start);
        let mut end: [u8;4] = [0;4];
        to.chars().nth(0).unwrap().encode_utf8(&mut end);

    
        assert!(dfa.find(" ").is_err());
        assert!(dfa.find("r").is_ok());
        assert!(dfa.find("s").is_ok());
        assert!(dfa.find("~").is_ok());
        //'ƒ'.encode_utf8(&mut temp);
        //println!("ƒ: {:?}",temp);
        assert!(dfa.find("|").is_ok());
        assert!(dfa.find("ƒ").is_ok()); // start:[114, 0, 0, 0] end:[194, 172, 0, 0]  ƒ: [198, 146, 0, 0]
        

        assert!(dfa.find("ƒ").is_ok());
        assert!(dfa.find("®").is_ok());
        assert!(dfa.find("«").is_ok());
        assert!(dfa.find("ª").is_ok());
        assert!(dfa.find("z").is_ok());
    


        // 'ॉ'.encode_utf8(&mut temp);
        // println!("ॉ: {:?}",temp);

        // 'ॐ'.encode_utf8(&mut temp);
        // println!("ॐ: {:?}",temp);

        // 'ख़'.encode_utf8(&mut temp);
        // println!("ख़: {:?}",temp);

        assert!(dfa.find("ॉ").is_ok());
        assert!(dfa.find("ऽ").is_ok());
        assert!(dfa.find("ऴ").is_ok());
        assert!(dfa.find("ॐ").is_ok());
        assert!(dfa.find("ख़").is_ok());

        assert!(dfa.find("🤖").is_ok());
        assert!(dfa.find("🟣").is_ok());
        assert!(dfa.find("🗨").is_ok());
        assert!(dfa.find("👵").is_ok());
        assert!(dfa.find("🎃").is_ok());
        assert!(dfa.find("🌶").is_ok());
        assert!(dfa.find("🌐").is_ok());
        assert!(dfa.find("䒗").is_ok());
        assert!(dfa.find("").is_ok());
        assert!(dfa.find("ϲ").is_ok());
        assert!(dfa.find("ϫ").is_ok());
        assert!(dfa.find("ζ").is_ok());
        assert!(dfa.find("Θ").is_ok());


        assert!(dfa.find("#").is_err());
        assert!(dfa.find("1").is_err());
        assert!(dfa.find("8").is_err());
        assert!(dfa.find("`").is_err());
        assert!(dfa.find("🤢").is_err()); //One after clown
        
    }

    

}