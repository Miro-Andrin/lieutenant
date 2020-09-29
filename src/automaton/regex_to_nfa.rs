use std::ops::Range;

use regex_syntax::{hir::ClassUnicodeRange, Parser};

use super::{ByteClass, NFA};
use anyhow::{bail, Result};

impl From<&ClassUnicodeRange> for NFA {
    fn from(rng: &ClassUnicodeRange) -> Self {
        let start = rng.start();
        let end = rng.end();

        if start as u32 == 0x0 && end as u32 == 0x9 {
            //This means we are matching on a dot

            //Create a nfa the matches any byte that does not end in a 1.
            let nfa_not_end = NFA::empty();

            // let mut buffer = [0; 4];
            //     let mut classes = vec![ByteClass::empty(); 4];
            //     for c in one_of.chars() {
            //         let bytes = c.encode_utf8(&mut buffer);
            //         for (i, b) in bytes.bytes().enumerate() {
            //             if i + 1 < c.len_utf8() {
            //                 classes[i][b] = 2;
            //             } else {
            //                 classes[i][b] = 1;
            //             }
            //         }
            //     }
            //     let mut nfa = NFA::empty();
            //     let mut id = nfa.start;

            //     let classes: Vec<_> = classes
            //         .into_iter()
            //         .take_while(|class| !class.is_empty())
            //         .collect();

            //     let end = StateId::of(classes.len() as u32);

            //     for class in classes {
            //         let next_id = nfa.push_state();
            //         if next_id == end {
            //             nfa.set_transitions(id, class, vec![vec![], vec![end], vec![]])
            //         } else {
            //             nfa.set_transitions(id, class, vec![vec![], vec![end], vec![next_id]]);
            //         }
            //         id = next_id;
            //     }

            //     nfa.end = id;

            //     nfa
        }

        //
        println!("{:?}", rng);
        todo!();
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
