use fnv::FnvHashMap;

type StateId = usize;

struct NFA {
    states: Vec<FnvHashMap<char, Vec<StateId>>>,
}