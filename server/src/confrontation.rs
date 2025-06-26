use std::sync::Arc;

use crate::agent::Agent;


pub struct Confrontation {
    pub ordered_player: Vec<Arc<Agent>>,
}