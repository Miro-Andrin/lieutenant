mod byteclass;
mod nfa;
mod nfa_ops;
mod regex;
mod state;

#[cfg(test)]
mod quickcheck;

pub use nfa::NFA;
pub use nfa_ops::*;

pub enum RegexError {
    // It does not make sense for the regex to contain the following symbols,
    // and we also dont suport them.
    StartLine,
    EndLine,
    StartText,
    EndText,

    // The following regex features we could possibly add suport for in the future,
    // but its not a high priority atm.
    WordBoundaryUnicode,
    WordBoundaryUnicodeNegate,
    WordBoundaryAscii,
    WordBoundaryAsciiNegate,

    // This just means what you gave was not valid regex.
    RegexParseError(regex_syntax::Error),
}

impl std::fmt::Debug for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RegexError::StartLine => {
                write!(f,"We don't suprt start line in regex, so ^ is not allowed in the regex without escaping it with \\.")
            }
            RegexError::EndLine => {
                write!(f,"We don't suport end line in regex, so $ is not allowed in the regex  without escaping it with \\.")
            }
            RegexError::StartText => {
                write!(f,"We don't suport start text symbol in regex, so \\A (or ^ in the beginning) is not allowed in the regex.")
            }
            RegexError::EndText => {
                write!(f,"We don't suport end of text symbol in regex, so \\z is not allowed in the regex.")
            }
            RegexError::WordBoundaryUnicode => {
                write!(
                    f,
                    "We don't suport unicode world boundary, so \\b is not allowed in the regex."
                )
            }
            RegexError::WordBoundaryUnicodeNegate => {
                write!(f,"We don't suport \"not a unicode world boundary\", so \\B is not allowed in the regex.")
            }
            RegexError::WordBoundaryAscii => {
                write!(f,"We don't suport \"not a unicode world boundary\", so (?-u:\\b) is not allowed in the regex.")
            }
            RegexError::WordBoundaryAsciiNegate => {
                write!(f,"We don't suport \"not a unicode world boundary\", so (?-u:\\B) is not allowed in the regex.")
            }

            RegexError::RegexParseError(err) => std::fmt::Display::fmt(&err, f),
        }
    }
}

#[derive(Debug)]
pub enum AutomataBuildError {
    // Something is wrong with the automatas regex. It is using a feature we dont suport.
    RegexError(RegexError),

    // Note: 'utf8_range_to_nfa.rs' assumes this is the only error case one can get
    // from building a nfa by hand. If any error case is added consider if it affects
    // those assumptions.
    RanOutOfByteClassIds,
}
