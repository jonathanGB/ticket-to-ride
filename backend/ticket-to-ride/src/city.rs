use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::Display;

/// All the different cities on the map.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize_repr,
    Display,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize_repr,
)]
#[repr(u8)]
pub enum City {
    Atlanta = 1,
    Boston = 2,
    Calgary = 3,
    Charleston = 4,
    Chicago = 5,
    Dallas = 6,
    Denver = 7,
    Duluth = 8,
    #[strum(serialize = "El Paso")]
    ElPaso = 9,
    Helena = 10,
    Houston = 11,
    #[strum(serialize = "Kansas City")]
    KansasCity = 12,
    #[strum(serialize = "Las Vegas")]
    LasVegas = 13,
    #[strum(serialize = "Little Rock")]
    LittleRock = 14,
    #[strum(serialize = "Los Angeles")]
    LosAngeles = 15,
    Miami = 16,
    #[strum(serialize = "Montréal")]
    Montreal = 17,
    Nashville = 18,
    #[strum(serialize = "New Orleans")]
    NewOrleans = 19,
    #[strum(serialize = "New York")]
    NewYork = 20,
    #[strum(serialize = "Oklahoma City")]
    OklahomaCity = 21,
    Omaha = 22,
    Phoenix = 23,
    Pittsburgh = 24,
    Portland = 25,
    Raleigh = 26,
    #[strum(serialize = "Saint Louis")]
    SaintLouis = 27,
    #[strum(serialize = "Salt Lake City")]
    SaltLakeCity = 28,
    #[strum(serialize = "San Francisco")]
    SanFrancisco = 29,
    #[strum(serialize = "Santa Fe")]
    SantaFe = 30,
    #[strum(serialize = "Sault St. Marie")]
    SaultStMarie = 31,
    Seattle = 32,
    Toronto = 33,
    Vancouver = 34,
    Washington = 35,
    Winnipeg = 36,
}

/// Top-level representation of a connection between two cities.
pub type CityToCity = (City, City);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_city_to_string() {
        assert_eq!(City::Atlanta.to_string(), "Atlanta");
        assert_eq!(City::Raleigh.to_string(), "Raleigh");
    }

    #[test]
    fn complex_city_to_string() {
        assert_eq!(City::ElPaso.to_string(), "El Paso");
        assert_eq!(City::KansasCity.to_string(), "Kansas City");
        assert_eq!(City::LasVegas.to_string(), "Las Vegas");
        assert_eq!(City::LittleRock.to_string(), "Little Rock");
        assert_eq!(City::LosAngeles.to_string(), "Los Angeles");
        assert_eq!(City::Montreal.to_string(), "Montréal");
        assert_eq!(City::NewOrleans.to_string(), "New Orleans");
        assert_eq!(City::NewYork.to_string(), "New York");
        assert_eq!(City::OklahomaCity.to_string(), "Oklahoma City");
        assert_eq!(City::SaintLouis.to_string(), "Saint Louis");
        assert_eq!(City::SaltLakeCity.to_string(), "Salt Lake City");
        assert_eq!(City::SanFrancisco.to_string(), "San Francisco");
        assert_eq!(City::SantaFe.to_string(), "Santa Fe");
        assert_eq!(City::SaultStMarie.to_string(), "Sault St. Marie");
    }

    #[test]
    fn city_to_json() -> serde_json::Result<()> {
        assert_eq!(serde_json::to_string(&City::Duluth)?, "8");
        assert_eq!(serde_json::to_string(&City::Montreal)?, "17");

        Ok(())
    }

    #[test]
    fn json_to_city() -> serde_json::Result<()> {
        assert_eq!(serde_json::from_str::<City>("16")?, City::Miami);
        assert_eq!(serde_json::from_str::<City>("23")?, City::Phoenix);

        Ok(())
    }

    #[test]
    fn invalid_json_to_city() {
        assert!(serde_json::from_str::<City>("0").is_err());
    }
}
