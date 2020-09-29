use std::ops::{Range, RangeInclusive};

use regex_syntax::{hir::ClassUnicodeRange, Parser};

use super::{ByteClass, StateId, NFA};
use anyhow::{bail, Result};

//TODO defman make sure this code actually works. I dont remeber if i copoed it from pattern and never fixed it,
// or if you wrote it.
impl From<Range<u8>> for NFA {
    fn from(range: Range<u8>) -> Self {
        let mut buffer = [0; 4];
        let mut classes = vec![ByteClass::empty(); 4];
        for c in range {
            let bytes = char::from(c).encode_utf8(&mut buffer);
            for (i, b) in bytes.bytes().enumerate() {
                if i + 1 < char::from(c).len_utf8() {
                    classes[i][b] = 2;
                } else {
                    classes[i][b] = 1;
                }
            }
        }

        let mut nfa = NFA::empty();
        let mut id = nfa.start;

        let classes: Vec<_> = classes
            .into_iter()
            .take_while(|class| !class.is_empty())
            .collect();

        let end = StateId::of(classes.len() as u32);

        for class in classes {
            let next_id = nfa.push_state();
            if next_id == end {
                nfa.set_transitions(id, class, vec![vec![], vec![end], vec![]])
            } else {
                nfa.set_transitions(id, class, vec![vec![], vec![end], vec![next_id]]);
            }
            id = next_id;
        }

        nfa.end = id;
        nfa
    }
}

impl From<RangeInclusive<u8>> for NFA {
    fn from(_: RangeInclusive<u8>) -> Self {
        todo!()
    }
}

/// Returns a NFA that matches any utf-8 char of length n (were n is one of 1,2,3 or 4)
/// by length i mean how many bytes you need to encode it.
/// any_un_unicode(1) -> NFA that accepts any valid ascii.
fn any_un_unicode(n: usize) -> NFA {
    match n {
        1 => {
            // Matches a single byte in starting with a zero
            // that is every byte less then 128
            NFA::from(0u8..128u8)
        }

        2 => {
            //The first byte must start with 110
            //That is evey byte less then 224
            let nfa = NFA::from(0u8..224u8);

            //The next byte must start with 10
            //That is every byte less then 192
            nfa.concat(&NFA::from(0u8..192u8))
        }

        3 => {
            //The fist byte must start with 1110
            //That is every byte less then 240
            let nfa = NFA::from(0u8..240u8);

            //The next 2 bytes must start with 10
            //That is two bytes less then 192
            nfa.concat(&NFA::from(0u8..192u8).concat(&NFA::from(0u8..192u8)))
        }

        4 => {
            //The first byte must start with 11110
            //That is every byte less then 248

            let nfa = NFA::from(0u8..248u8);

            //The next 3 bytes must start with 10
            //That is tree byte less then 248
            nfa.concat(
                &NFA::from(0u8..192u8)
                    .concat(&NFA::from(0u8..192u8))
                    .concat(&NFA::from(0u8..192u8)),
            )
        }

        _ => unreachable!(),
    }
}

/// Takes a char and returns a NFA that matches any char of the same encoding lenght that is less then (inclusive)
/// the given char.
fn any_un_lt_eq(c: char) -> NFA {
    let mut buffer = [0u8; 4];
    c.encode_utf8(&mut buffer);

    let mut nfa = NFA::empty();
    for i in 0..buffer.len() {
        nfa = nfa.concat(&NFA::from(0..=buffer[i]));
    }
    nfa
}

/// Takes a char and returns a NFA that matches any char of the same encoding lenght that is greater then (inclusive)
/// the given char.
fn any_un_gt_eq(c: char) -> NFA {
    let mut buffer = [0u8; 4];
    c.encode_utf8(&mut buffer);

    match c.len_utf8() {
        1 => {
            //The first value must be less then 128 and greater then or equal to c as u8
            NFA::from(buffer[0]..128)
        }
        2 => {
            //The first value must be less then 224 (start with 110)
            let nfa = NFA::from(buffer[0]..224u8);
            //The second byte must less then 192  start with 10
            nfa.concat(&NFA::from(buffer[1]..192))
        }
        3 => {
            let mut nfa = NFA::from(buffer[0]..240u8);
            //The second and third byte must less then 192  start with 10
            nfa.concat(&NFA::from(buffer[1]..192).concat(&NFA::from(buffer[2]..192)))
        }
        4 => {
            let mut nfa = NFA::from(buffer[0]..240u8);
            //The second and third byte must less then 192  start with 10
            nfa.concat(
                &NFA::from(buffer[1]..192)
                    .concat(&NFA::from(buffer[2]..192).concat(&NFA::from(buffer[3]..192))),
            )
        }
        _ => unreachable!(),
    }
}

/// This returns a nfa that matches all char's between (inclusive the start and end)
/// start and end have same length, by length i mean how many bytes you need to encode it.
fn any_between(start: char, end: char) -> NFA {
    let mut start_buffer = [0u8; 4];
    let mut end_buffer = [0u8; 4];
    start.encode_utf8(&mut start_buffer);
    end.encode_utf8(&mut end_buffer);

    let mut nfa = NFA::empty();
    for i in 1..start.len_utf8() {
        nfa = nfa.concat(&NFA::from(
            start_buffer[i as usize] as u8..=end_buffer[i as usize] as u8,
        ));
    }
    nfa
}

impl From<&ClassUnicodeRange> for NFA {
    fn from(rng: &ClassUnicodeRange) -> Self {
        let start = rng.start();
        let end = rng.end();

        let start_byte_len = start.len_utf8();
        let end_byte_len = end.len_utf8();

        if start_byte_len == end_byte_len {
            return any_between(start, end);
        }

        let mut nfa = NFA::empty();

        if start_byte_len + 1 < end_byte_len {
            //These unicode char lengths are compleatly matched, that is to say
            //every char of length n should be accepted.
            for n in start_byte_len + 1..end_byte_len {
                nfa = nfa.union(&any_un_unicode(n));
            }
        }

        nfa.union(&any_un_lt_eq(end).union(&any_un_gt_eq(start)))
    }
}

pub(crate) fn regex_to_nfa(regex: &str) -> Result<NFA> {
    let hir = Parser::new().parse(regex)?;
    hir_to_nfa(hir)
}

fn hir_to_nfa(hir: regex_syntax::hir::Hir) -> Result<NFA> {
    match hir.into_kind() {
        regex_syntax::hir::HirKind::Empty => Ok(NFA::single_u8()),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(uni) => Ok(NFA::literal(&uni.to_string())),
            regex_syntax::hir::Literal::Byte(byte) => Ok(NFA::literal(&byte.to_string())),
        },
        regex_syntax::hir::HirKind::Class(class) => {
            match class {
                regex_syntax::hir::Class::Unicode(uni) => {
                    let mut nfa = NFA::empty();
                    let mut states = [[0u8; 256]; 4];
                    for range in uni.ranges() {
                        let mut start = [0u8; 4];
                        range.start().encode_utf8(&mut start);
                        let mut end = [0u8; 4];
                        range.end().encode_utf8(&mut end);

                        // [0x00, 0x00, 0x00, 0x09]
                        // [0x00, 0x10, 0xff, 0xff]

                        for (state, (lower, upper)) in
                            start.iter().copied().zip(end.iter().copied()).enumerate()
                        {
                            for b in lower..=upper {
                                if b < 127 {
                                    states[state][b as usize] = 1;
                                } else {
                                    states[state][b as usize] = 2;
                                }
                            }
                        }
                    }
                    todo!()
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
                let nfa = hir_to_nfa(*x.hir)?;
                Ok(nfa.repeat())
            } else {
                bail!("We dont suport non greedy patterns")
            }
        }
        regex_syntax::hir::HirKind::Group(group) => {
            //TODO i dont know how we are suposed to interprite an empty
            //hir/nfa in this case. Should it maybe be a no-op?
            hir_to_nfa(*group.hir)
        }
        regex_syntax::hir::HirKind::Concat(cats) => {
            let mut nfas = cats.iter().map(|hir| hir_to_nfa(hir.to_owned()));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst = fst.concat(&nfa?);
            }
            Ok(fst)
        }

        regex_syntax::hir::HirKind::Alternation(alts) => {
            let mut nfas = alts.iter().map(|hir| hir_to_nfa(hir.to_owned()));
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
    fn test_from_range() {
        for u1 in 0u8..=255 {
            for u2 in u1..=255 {

                let nfa = NFA::from(u1..u2);
                let dfa = DFA::from(nfa);

                for u3 in 0u8..=255 {

                    if u1 <= u3 && u3 < u2 {
                        if !(dfa.find(&char::from(u3).to_string()).is_ok()) {
                            println!("range {} to {} non inclusive does not contain {}!",u1,u2,u3);
                            assert!(false);
                        }
                    } else {
                        if !(dfa.find(&char::from(u3).to_string()).is_err()){
                            println!("range {} to {} non inclisive contains {}!",u1,u2,u3);
                            assert!(false);
                        }
                    }

                }
            }

        }
    }


    #[test]
    fn test_from_inclusive_range() {
        for u1 in 0u8..=255 {
            for u2 in u1..=255 {

                let nfa = NFA::from(u1..u2);
                let dfa = DFA::from(nfa);

                for u3 in 0u8..=255 {

                    if u1 <= u3 && u3 <= u2 {
                        if !(dfa.find(&char::from(u3).to_string()).is_ok()) {
                            println!("range {} to {} inclusive does not contain {}!",u1,u2,u3);
                            assert!(false);
                        }
                    } else {
                        if !(dfa.find(&char::from(u3).to_string()).is_err()){
                            println!("range {} to {} inclisive contains {}!",u1,u2,u3);
                            assert!(false);
                        }
                    }

                }
            }

        }
    }

}