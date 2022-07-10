use crate::{card::CardDealer, game_phase::GamePhase, map::Map, player::Player};

use smallvec::SmallVec;
const MAX_PLAYERS: usize = 5;

pub struct Manager {
    phase: GamePhase,
    turn: u32,
    map: Map,
    card_dealer: CardDealer,
    players: SmallVec<[Player; MAX_PLAYERS]>,
}

impl Manager {
    pub fn get_state(&self, player_id: usize) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
