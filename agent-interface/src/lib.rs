pub trait Agent<State, Action> {
    fn init() -> Self;

    //State : Serializable ? string ?
    fn select_action(&mut self, state: State) -> Option<Action>;
}

pub trait Game<State, Action> {
    fn init(&mut self);
    fn apply_action(&mut self, action: &Action) -> Result<(), ()>; //non mutable ?, -> bool ?
    fn is_finished(&self) -> bool;
    fn number_of_players(&self) -> u32; //self necessary for vtable
    fn is_deterministic(&self) -> bool;

    //Supposed true for now
    //fn is_perfect_information(&self) -> bool;  //*has* ?

    //Supposed true for now
    // fn is_sequential(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyGame {}

    impl Game<(), ()> for DummyGame {
        fn init(&mut self) {}

        fn apply_action(&mut self, _action: &()) -> Result<(), ()> {
            Ok(())
        }

        fn is_finished(&self) -> bool {
            false
        }

        fn number_of_players(&self) -> u32 {
            0
        }

        fn is_deterministic(&self) -> bool {
            true
        }
    }

    #[test]
    fn it_works() {
        let game: Box<dyn Game<(),()>> = Box::new(DummyGame{});
        assert!(game.is_deterministic());
    }
}
