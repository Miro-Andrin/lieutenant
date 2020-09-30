use std::ops::Range;

use regex_syntax::{hir::ClassUnicodeRange, Parser};

use super::{ByteClass, StateId, NFA};
use anyhow::{bail, Result};

pub(crate) fn regex_to_nfa(regex: &str) -> Result<NFA> {
    let hir = Parser::new().parse(regex)?;
    hir_to_nfa(&hir)
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
                    let mut classes = [[0u8; 256]; 4];
                    for range in uni.ranges() {
                        let mut start = [0u8; 4];
                        range.start().encode_utf8(&mut start);
                        let mut end = [0u8; 4];
                        range.end().encode_utf8(&mut end);

                        println!("START_BUFFER: {:?}",start);
                        println!("END_BUFFER: {:?}",end);

                        let mut bytes = start.iter().copied().zip(end.iter().copied()).enumerate();

                        let (c, (lower, upper)) = bytes.next().unwrap();
                        for b in lower..=upper {
                            if b < 128 {
                                classes[c][b as usize] = 1;
                            } else if b >= 192 && b < 224 {
                                classes[c][b as usize] = 2;
                            } else if b >= 224 && b < 240 {
                                classes[c][b as usize] = 3;
                            } else if b >= 240 && b < 248 {
                                classes[c][b as usize] = 4;
                                println!("240<=B<=248: {}",b);
                            }
                        }

                        for (c, (lower, upper)) in bytes {
                            for b in lower..=upper {
                                if b < 192 {
                                    classes[c][b as usize] = 1;
                                    println!("B<192: {}",b);
                                }
                            }
                        }
                    }

                    let mut nfa = NFA::empty();
                    nfa.end = nfa.push_state();
                    let mut prev = nfa.end;
                    let mut start_table = vec![vec![], vec![nfa.end]];                    


                    //let states = *classes[0].iter().max().unwrap();
                    let states = 4;
                    for i in 1..states {
                        let cur = nfa.push_state();
                        nfa.set_transitions(
                            cur,
                            ByteClass(classes[i as usize].to_vec()),
                            vec![vec![], vec![prev]],
                        );
                        prev = cur;

                        //Setting transition for first node
                        start_table.push(vec![cur]);
                    }

                    nfa.set_transitions(nfa.start, ByteClass(classes[0].to_vec()), start_table);

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
