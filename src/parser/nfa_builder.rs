fn nfa_builder<E: Tuple, S: Defalt, P: Parser<E,S>>(p: P, command_id: usize) -> NFA::<usize> {
    let mut nfa = NFA::<usize>::regex(p.regex());
    nfa.assosiate_ends(command_id);
    nfa
}