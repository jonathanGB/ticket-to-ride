use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Serialize, Deserialize, Display, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TrainColor {
  Black,
  Blue,
  Green,
  Orange,
  Pink,
  Red,
  White,
  Wild,
  Yellow,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn train_color_to_string() {
    assert_eq!(TrainColor::Orange.to_string(), "orange");
    assert_eq!(TrainColor::Pink.to_string(), "pink");
  }

  #[test]
  fn train_color_to_json() -> serde_json::Result<()> {
    assert_eq!(serde_json::to_string(&TrainColor::Blue)?, r#""blue""#);
    assert_eq!(serde_json::to_string(&TrainColor::Red)?, r#""red""#);
    Ok(())
  }

  #[test]
  fn json_to_train_color() -> serde_json::Result<()> {
    assert_eq!(
      serde_json::from_str::<TrainColor>(r#""wild""#)?,
      TrainColor::Wild
    );
    assert_eq!(
      serde_json::from_str::<TrainColor>(r#""green""#)?,
      TrainColor::Green
    );

    Ok(())
  }

  #[test]
  fn invalid_json_to_train_color() {
    assert!(serde_json::from_str::<TrainColor>(r#""turquoise""#).is_err());
  }
}
