#[derive(PartialEq,Eq,Debug,Clone,Copy)]
pub enum Deterministicness {
    Deterministic,
    NonDeterministic,
}

/// Sequential: one player after the other (chess)
/// Simultaneous: everyone play at the same time (rock-papers-scissors)
#[derive(PartialEq,Eq,Debug,Clone,Copy)]
pub enum Sequentialness {
    Sequential,
    Simultaneous,
}

#[derive(PartialEq,Eq,Debug,Clone,Copy)]
pub enum Information {
    PerfectInformation,
    PartialInformation,
}

#[derive(PartialEq,Eq,Debug,Clone,Copy)]
pub struct GameInfo {
    pub num_player : u32,
    pub deterministicness : Deterministicness,
    pub sequentialness : Sequentialness,
    pub information : Information,
    // symmetric ? (AgentA > AgentB </=> AgentB < AgentA)
}
