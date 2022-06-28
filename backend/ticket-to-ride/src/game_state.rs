use crate::city::City;
use crate::game_phase::GamePhase;
use crate::train_color::TrainColor;

pub struct Route {
  start: City,
  end: City,
  color: Option<TrainColor>,
  length: u8,
}

pub enum DestinationCard {}

pub struct GameState {
  phase: GamePhase,
  turn: u32,
}

#[cfg(test)]
mod tests {}
