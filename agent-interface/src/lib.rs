pub trait Agent<State,Action> {
    fn init() -> Self;

    //State : serializable ? string ?
    fn select_action(&mut self, state: State) -> Option<Action>;
}

pub trait Game<State,Action> {
    fn init() -> Self;
    fn apply_action(&mut self, action : &Action) -> Result<(),()>;
    fn is_finished(&self) -> bool;
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
