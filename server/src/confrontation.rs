use std::{fmt::Display, sync::Arc};

use crate::agent::Agent;

#[derive(Debug, Clone)]
pub struct Confrontation {
    pub ordered_player: Vec<Arc<Agent>>,
}

impl Display for Confrontation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .ordered_player
            .iter()
            .fold(String::new(), |acu, agent| {
                if acu.is_empty() {
                    acu + &agent.name
                } else {
                    acu + " VS " + &agent.name
                }
            });
        write!(f, "[{s}]")
    }
}
