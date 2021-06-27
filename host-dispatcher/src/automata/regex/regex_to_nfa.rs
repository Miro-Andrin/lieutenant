/*
We use the regex crates fantastic frontend to convert regex to their "HIR" and from that we lower
into nfa. This makes it so that we suport most of the features of the regex crate,
but we dont need to do the tricky parsing of regex syntax.
*/

use regex_syntax::Parser;

use crate::automata::{AutomataBuildError, RegexError, NFA};

fn regex_to_nfa(regex: &str) -> Result<NFA, AutomataBuildError> {
    let hir = match Parser::new().parse(regex) {
        Ok(hir) => Ok(hir),
        Err(err) => Err(AutomataBuildError::RegexError(RegexError::RegexParseError(
            err,
        ))),
    };
    hir_to_nfa(&hir?)
}

fn repeated_n_times(mut nfa: NFA, n: u32) -> Result<NFA, AutomataBuildError> {
    let mut result = NFA::with_capacity(nfa.states.len(), nfa.translations.len(), nfa.ends.len());
    let mut accounted = 0_u32;

    for bit_index in 0..=31 {
        if accounted == n {
            break;
        }

        if n & (1 << bit_index) != 0 {
            result.followed_by(nfa.clone())?;
            accounted |= 1 << bit_index;
        }

        // Double nfa in size
        nfa.followed_by(nfa.clone())?;
    }

    Ok(result)
}

fn hir_to_nfa(hir: &regex_syntax::hir::Hir) -> Result<NFA, AutomataBuildError> {
    match hir.kind() {
        regex_syntax::hir::HirKind::Empty => Ok(NFA::literal("")?),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(uni) => Ok(NFA::literal(&uni.to_string())?),
            regex_syntax::hir::Literal::Byte(byte) => Ok(NFA::literal(&byte.to_string())?),
        },
        regex_syntax::hir::HirKind::Class(class) => {
            match class {
                regex_syntax::hir::Class::Unicode(uni) => {
                    // This initial capacity is just plain wrong.
                    let mut nfa = NFA::with_capacity(
                        5 * uni.ranges().len(),
                        uni.ranges().len(),
                        uni.ranges().len(),
                    );
                    for range in uni.ranges() {
                        nfa = nfa.union(NFA::from(range))?;
                    }
                    Ok(nfa)
                }
                regex_syntax::hir::Class::Bytes(byte) => {
                    let mut nfa = NFA::with_capacity(
                        byte.ranges().len() * 2,
                        byte.ranges().len(),
                        byte.ranges().len(),
                    );
                    for range in byte.iter() {
                        let mut range_nfa = NFA::with_capacity(2, 1, 1);
                        let a = range_nfa.push_state();
                        let b = range_nfa.push_state();
                        // TODO: This reads as a bug to me.
                        range_nfa.push_end(b);
                        //
                        range_nfa.push_connections(a, b, range.start()..=range.end())?;
                        nfa = nfa.union(range_nfa)?;
                    }
                    Ok(nfa)
                }
            }
        }
        regex_syntax::hir::HirKind::Anchor(x) => match x {
            regex_syntax::hir::Anchor::StartLine => {
                Err(AutomataBuildError::RegexError(RegexError::StartLine))
            }
            regex_syntax::hir::Anchor::EndLine => {
                Err(AutomataBuildError::RegexError(RegexError::EndLine))
            }
            regex_syntax::hir::Anchor::StartText => {
                Err(AutomataBuildError::RegexError(RegexError::StartText))
            }
            regex_syntax::hir::Anchor::EndText => {
                Err(AutomataBuildError::RegexError(RegexError::EndText))
            }
        },
        regex_syntax::hir::HirKind::WordBoundary(boundary) => match boundary {
            regex_syntax::hir::WordBoundary::Unicode => {
                return Err(AutomataBuildError::RegexError(
                    RegexError::WordBoundaryUnicode,
                ));
            }
            regex_syntax::hir::WordBoundary::UnicodeNegate => {
                return Err(AutomataBuildError::RegexError(
                    RegexError::WordBoundaryUnicodeNegate,
                ));
            }
            regex_syntax::hir::WordBoundary::Ascii => {
                return Err(AutomataBuildError::RegexError(
                    RegexError::WordBoundaryAscii,
                ));
            }
            regex_syntax::hir::WordBoundary::AsciiNegate => {
                return Err(AutomataBuildError::RegexError(
                    RegexError::WordBoundaryAsciiNegate,
                ));
            }
        },
        regex_syntax::hir::HirKind::Repetition(x) => {
            match &x.kind {
                regex_syntax::hir::RepetitionKind::ZeroOrOne => {
                    let mut nfa: NFA = hir_to_nfa(&x.hir)?; // TODO check
                    nfa.make_optional();
                    Ok(nfa)
                }
                regex_syntax::hir::RepetitionKind::ZeroOrMore => hir_to_nfa(&x.hir)?.repeat(),
                regex_syntax::hir::RepetitionKind::OneOrMore => {
                    let nfa = hir_to_nfa(&x.hir)?;
                    let mut fst = nfa.clone();
                    fst.followed_by(nfa.repeat()?)?;
                    Ok(fst)
                }
                regex_syntax::hir::RepetitionKind::Range(range) => match range {
                    // We dont care about greedy vs lazy ranges, since we only use the regex to detect
                    // fullmatch, and dont care about matchgroups.
                    regex_syntax::hir::RepetitionRange::Exactly(exact) => {
                        let nfa = hir_to_nfa(&x.hir)?;
                        repeated_n_times(nfa, *exact)
                    }
                    regex_syntax::hir::RepetitionRange::AtLeast(exact) => {
                        let nfa = hir_to_nfa(&x.hir)?;
                        let mut result = repeated_n_times(nfa.clone(), *exact)?;
                        result.followed_by(nfa.repeat()?)?;
                        Ok(result)
                    }
                    regex_syntax::hir::RepetitionRange::Bounded(n, m) => {
                        let org = hir_to_nfa(&x.hir)?;
                        let mut nfa = repeated_n_times(org.clone(), *n)?;

                        let cap = if (*m as i64) - (*n as i64) > 0 {
                            (m - n) as usize
                        } else {
                            1
                        };

                        let mut result = NFA::with_capacity(
                            cap * nfa.states.len(),
                            nfa.states.len(),
                            nfa.ends.len(),
                        );

                        for _ in *n..=*m {
                            result = result.union(nfa.clone())?;
                            nfa.followed_by(org.clone())?;
                        }

                        Ok(result)
                    }
                },
            }
        }
        regex_syntax::hir::HirKind::Group(group) => match &group.kind {
            regex_syntax::hir::GroupKind::CaptureIndex(_) => hir_to_nfa(&group.hir),
            regex_syntax::hir::GroupKind::NonCapturing => hir_to_nfa(&group.hir),
            regex_syntax::hir::GroupKind::CaptureName { name: _, index: _ } => {
                hir_to_nfa(&group.hir)
            }
        },
        regex_syntax::hir::HirKind::Concat(cats) => {
            let mut nfas = cats.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst.followed_by(nfa?)?;
            }
            Ok(fst)
        }
        regex_syntax::hir::HirKind::Alternation(alts) => {
            let mut nfas = alts.iter().map(|hir| hir_to_nfa(hir));
            let mut fst = nfas.next().unwrap()?;
            for nfa in nfas {
                fst = fst.union(nfa?)?;
            }
            Ok(fst)
        }
    }
}

/// Checks if regex contains a feature we don't suport, or it just cant be parsed as
/// valid regex. If this test passes for a command, then the only other failurecase for
/// creating a nfa is running out of StateId: u32.
pub fn we_suport_regex(regex: &str) -> Result<(), RegexError> {
    let hir = Parser::new().parse(regex);
    let hir = match hir {
        Ok(x) => x,
        Err(e) => return Err(RegexError::RegexParseError(e)),
    };

    we_suport_hir(&hir)
}

/// Returns wheter or not we are able to parse regex. We dont suport certain features.
pub(crate) fn we_suport_hir(hir: &regex_syntax::hir::Hir) -> Result<(), RegexError> {
    match hir.kind() {
        regex_syntax::hir::HirKind::Empty => Ok(()),
        regex_syntax::hir::HirKind::Literal(lit) => match lit {
            regex_syntax::hir::Literal::Unicode(_) => Ok(()),
            regex_syntax::hir::Literal::Byte(_) => Ok(()),
        },
        regex_syntax::hir::HirKind::Class(class) => match class {
            regex_syntax::hir::Class::Unicode(_) => Ok(()),
            regex_syntax::hir::Class::Bytes(_) => Ok(()),
        },
        regex_syntax::hir::HirKind::Anchor(x) => match x {
            regex_syntax::hir::Anchor::StartLine => Err(RegexError::StartLine),
            regex_syntax::hir::Anchor::EndLine => Err(RegexError::EndLine),
            regex_syntax::hir::Anchor::StartText => Err(RegexError::StartText),
            regex_syntax::hir::Anchor::EndText => Err(RegexError::EndText),
        },
        regex_syntax::hir::HirKind::WordBoundary(boundary) => match boundary {
            regex_syntax::hir::WordBoundary::Unicode => Err(RegexError::WordBoundaryUnicode),
            regex_syntax::hir::WordBoundary::UnicodeNegate => {
                Err(RegexError::WordBoundaryUnicodeNegate)
            }
            regex_syntax::hir::WordBoundary::Ascii => Err(RegexError::WordBoundaryAscii),
            regex_syntax::hir::WordBoundary::AsciiNegate => {
                Err(RegexError::WordBoundaryAsciiNegate)
            }
        },
        regex_syntax::hir::HirKind::Repetition(x) => {
            return we_suport_hir(&x.hir)
        }
        regex_syntax::hir::HirKind::Group(group) =>  {
            return we_suport_hir(&group.hir)
        },
        regex_syntax::hir::HirKind::Concat(cats) => {
            for meow in cats {
                if let Err(x) = we_suport_hir(meow) {
                    return Err(x);
                }
            }
            Ok(())
        }
        regex_syntax::hir::HirKind::Alternation(alts) => {
            for alt in alts {
                if let Err(x) = we_suport_hir(alt) {
                    return Err(x);
                }
            }
            Ok(())
        }
    }
}

// Converts regex to nfa.
impl NFA {
    pub fn regex(string: &str) -> Result<NFA, AutomataBuildError> {
        regex_to_nfa(string)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_we_dont_suport() {
        let regex = "^";
        assert!(we_suport_regex(regex).is_err());

        let regex = "$";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\A";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\z";
        assert!(we_suport_regex(regex).is_err());

        let regex = "\\b";
        assert!(we_suport_regex(regex).is_err());
    }

    #[test]
    fn simple1() {
        let nfa = NFA::regex("fu.*").unwrap();

        for case in &["funN", "fu.\"", "fu,-", "fu{:", "fut!"] {
            assert!(nfa.find(case).is_ok());
        }
    }

    #[test]
    fn simple2() {
        let nfa = NFA::regex("fu..*").unwrap();

        for case in &["funN", "fu.\"", "fu,-", "fu{:", "fut!"] {
            assert!(nfa.find(case).is_ok());
        }

        for case in &["fu"] {
            match nfa.find(case) {
                Ok(_) => {
                    panic!()
                }
                Err(_) => {}
            }
        }
    }

    #[test]
    fn digit() {
        let nfa = NFA::regex("\\d").unwrap();

        for case in &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
            let a = nfa.find(case);
            assert!(match a {
                Ok(x) => {
                    x.len() > 0
                }
                Err(_) => false,
            });
        }
        for case in &["a"] {
            assert!(match nfa.find(case) {
                Ok(x) => {
                    x.len() == 0
                }
                Err(_) => true,
            });
        }
    }

    #[test]
    fn not_digit() {
        let nfa = NFA::regex("\\D").unwrap();

        for case in &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
            assert!(nfa.find(case).is_err());
        }
        for case in &["a", "q"] {
            assert!(nfa.find(case).is_ok());
        }
    }

    #[test]
    fn direct_sibtraction() {
        let nfa = NFA::regex("[0-9--4]").unwrap();

        for case in &["1", "2", "3", "5", "6", "7", "8", "9", "0"] {
            assert!(nfa.find(case).is_ok());
        }
        for case in &["4", "a"] {
            assert!(nfa.find(case).is_err());
        }
    }

    #[test]
    fn repetitions_exact() {
        let regex = "a{5}";
        let nfa = NFA::regex(regex).unwrap();
        assert!(nfa.find("aaaaa").is_ok());
        assert!(nfa.find("aaaa").is_err());
        assert!(nfa.find("aaaaaa").is_err());
        assert!(nfa.find("").is_err());
    }

    #[test]
    fn repetitions_at_least() {
        let regex = "a{5,}";
        let nfa = NFA::regex(regex).unwrap();

        assert!(nfa.find("aaaaa").is_ok());
        assert!(nfa.find("aaaa").is_err());
        assert!(nfa.find("aaaaaa").is_ok());
        assert!(nfa.find("").is_err());
    }

    #[test]
    fn repetitions_between() {
        let regex = "a{5,8}";
        let nfa = NFA::regex(regex).unwrap();
        assert!(nfa.find("aaaaa").is_ok());
        assert!(nfa.find("aaaa").is_err());
        assert!(nfa.find("aaaaaa").is_ok());
        assert!(nfa.find("aaaaaaa").is_ok());
        assert!(nfa.find("aaaaaaaa").is_ok());
        assert!(nfa.find("aaaaaaaaa").is_err());
        assert!(nfa.find("").is_err());
    }

    #[test]
    fn repetition() {
        let mut nfa = NFA::regex("ho").unwrap();
        nfa = nfa.repeat().unwrap();

        assert!(nfa.find("").is_ok());
        assert!(nfa.find("ho").is_ok());
        assert!(nfa.find("hoho").is_ok());
        assert!(nfa.find("h").is_err());
    }

    #[test]
    fn repetition_lazy() {
        let nfa = NFA::regex("a{3,5}?").unwrap();

        assert!(nfa.find("aaa").is_ok());
        assert!(nfa.find("aaaa").is_ok());
        assert!(nfa.find("aaaaa").is_ok());
        assert!(nfa.find("").is_err());
    }
}
