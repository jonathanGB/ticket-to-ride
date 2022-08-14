use serde_repr::{Deserialize_repr, Serialize_repr};
#[allow(unused_imports)]
use strum::EnumCount;
use strum_macros::{Display, EnumCount as EnumCountMacro};

/// All the different cities on the map.
///
/// # JSON
/// Cities are serialized as an 8-bit unsigned integer.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize_repr,
    Display,
    EnumCountMacro,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize_repr,
)]
#[repr(u8)]
pub enum City {
    Atlanta = 0,
    Boston = 1,
    Calgary = 2,
    Charleston = 3,
    Chicago = 4,
    Dallas = 5,
    Denver = 6,
    Duluth = 7,
    #[strum(serialize = "El Paso")]
    ElPaso = 8,
    Helena = 9,
    Houston = 10,
    #[strum(serialize = "Kansas City")]
    KansasCity = 11,
    #[strum(serialize = "Las Vegas")]
    LasVegas = 12,
    #[strum(serialize = "Little Rock")]
    LittleRock = 13,
    #[strum(serialize = "Los Angeles")]
    LosAngeles = 14,
    Miami = 15,
    #[strum(serialize = "Montréal")]
    Montreal = 16,
    Nashville = 17,
    #[strum(serialize = "New Orleans")]
    NewOrleans = 18,
    #[strum(serialize = "New York")]
    NewYork = 19,
    #[strum(serialize = "Oklahoma City")]
    OklahomaCity = 20,
    Omaha = 21,
    Phoenix = 22,
    Pittsburgh = 23,
    Portland = 24,
    Raleigh = 25,
    #[strum(serialize = "Saint Louis")]
    SaintLouis = 26,
    #[strum(serialize = "Salt Lake City")]
    SaltLakeCity = 27,
    #[strum(serialize = "San Francisco")]
    SanFrancisco = 28,
    #[strum(serialize = "Santa Fe")]
    SantaFe = 29,
    #[strum(serialize = "Sault St. Marie")]
    SaultStMarie = 30,
    Seattle = 31,
    Toronto = 32,
    Vancouver = 33,
    Washington = 34,
    Winnipeg = 35,
}

/// Top-level representation of a connection between two cities.
pub type CityToCity = (City, City);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_count() {
        assert_eq!(City::COUNT, 36);
    }

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
        assert_eq!(serde_json::to_string(&City::Duluth)?, "7");
        assert_eq!(serde_json::to_string(&City::Montreal)?, "16");

        Ok(())
    }

    #[test]
    fn json_to_city() -> serde_json::Result<()> {
        assert_eq!(serde_json::from_str::<City>("15")?, City::Miami);
        assert_eq!(serde_json::from_str::<City>("22")?, City::Phoenix);

        Ok(())
    }

    #[test]
    fn invalid_json_to_city() {
        assert!(serde_json::from_str::<City>("36").is_err());
    }
}
