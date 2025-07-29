pub use anyhow;
use std::time::SystemTime;

/// What the game should implement
pub trait Game {
    type State;
    type Action;

    /// Apply an optional action to the game.
    ///
    /// `Option<Action>` is necessary because when `num_players >= 3`
    /// a player might be eliminated by not playing,
    /// but the game could still continue with the remaining players.
    ///
    /// # Error
    /// Returned when `action` is not valid (`action` is None, `action` is not allowed, ...).
    ///
    /// Even if `action` is not valid, `current_player` should be updated!
    fn apply_action(&mut self, action: &Option<Self::Action>) -> anyhow::Result<()>;

    /// The current state that will be given to the current player
    ///
    /// Does not returns &State because of annoying lifetime to deal with.
    fn get_state(&self) -> Self::State;

    /// The number of the player that should play now
    fn get_current_player_number(&self) -> usize;

    fn is_finished(&self) -> bool;

    fn get_player_score(&self, player_number: u32) -> f32;
}

/// What the agent should implement
pub trait Agent<G: Game> {
    fn init(&mut self);

    // State == String ? (Codingame-like)
    fn select_action(&mut self, state: G::State, deadline: SystemTime) -> G::Action;
}

/// What will be given to the evaluator to allow it to create games
pub trait GameFactory<G: Game> {
    /// Returns an initialized game
    fn new_game(&self) -> G;
}
