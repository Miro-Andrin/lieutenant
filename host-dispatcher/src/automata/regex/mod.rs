mod between;
mod regex_to_nfa;

pub use regex_to_nfa::we_suport_regex;
#[cfg(test)]
pub(crate) use regex_to_nfa::we_suport_hir;
