use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
  InLobby,
  Starting,
  Playing,
  LastTurn,
  Done,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn game_phase_to_json() -> serde_json::Result<()> {
    assert_eq!(serde_json::to_string(&GamePhase::InLobby)?, r#""in_lobby""#);
    assert_eq!(serde_json::to_string(&GamePhase::Playing)?, r#""playing""#);
    Ok(())
  }

  #[test]
  fn json_to_game_phase() -> serde_json::Result<()> {
    assert_eq!(
      serde_json::from_str::<GamePhase>(r#""last_turn""#)?,
      GamePhase::LastTurn
    );
    assert_eq!(
      serde_json::from_str::<GamePhase>(r#""done""#)?,
      GamePhase::Done
    );

    Ok(())
  }

  #[test]
  fn invalid_json_to_game_phase() {
    assert!(serde_json::from_str::<GamePhase>(r#""not_started""#).is_err());
  }
}
