use crate::city::City;
use crate::train_color::TrainColor;
use crate::train_color::TrainColor::*;
use smallvec::SmallVec;
use std::collections::HashMap;

/// Top-level representation of a connection between two cities.
/// Note that a `Route` from A to B is equal to a `Route` from B to A.
#[derive(Eq, Hash, Debug)]
pub struct Route(City, City);

impl From<(City, City)> for Route {
  fn from(tuple_route: (City, City)) -> Self {
    Self(tuple_route.0, tuple_route.1)
  }
}

impl PartialEq for Route {
  fn eq(&self, other: &Self) -> bool {
    if self.0 == other.0 {
      self.1 == other.1
    } else if self.0 == other.1 {
      self.1 == other.0
    } else {
      false
    }
  }
}

/// There can be multiple parallel routes between two cities.
/// `ParallelRoute` represents one of them, specifically its ownership and its color, if any.
#[derive(Debug, PartialEq)]
struct ParallelRoute {
  /// By whom this route is claimed, if any.
  claimed_by: Option<usize>,
  /// Whether this route has a specific color.
  train_color: Option<TrainColor>,
}

impl ParallelRoute {
  /// Returns a `ParalleRoute` with the given color.
  /// By default, a route is not claimed.
  fn new(train_color: Option<TrainColor>) -> Self {
    Self {
      claimed_by: None,
      train_color,
    }
  }
}

/// All (up to two) parallel routes between two cities.
#[derive(Debug, PartialEq)]
struct ParallelRoutes {
  /// Considering that we know that there can be up to two parallel routes between cities,
  /// we can optimize its storage on the stack thanks to `SmallVec`.
  parallel_routes: SmallVec<[ParallelRoute; 2]>,
  /// The distance between two cities. This is analogous to the number of train cards needed to claim the route.
  length: u8,
}

/// Convenience macro to generate a `ParallelRoutes`.
macro_rules! parallel_routes {
  ($l:literal, $($train_colors:expr),+) => {
    ParallelRoutes {
      parallel_routes: smallvec![$(ParallelRoute::new($train_colors)),+],
      length: $l,
    }
  };
}

/*
  Use cases:
    1. Claiming a route
      -> Must verify it's ok to claim it.
    2. See if a destination card is fulfilled.
      -> Need to BFS across claimed routes.
    3. Calculate longest route.
*/

/// The authoritative state of the map, per game.
/// This can be mutated as players claim routes throughout the game.
pub struct Map {
  /// Maps the concept of two cities being adjacent (i.e. having a `Route`) to the underlying
  /// parallel routes between the two.
  /// Note that thanks to `Route`'s custom implementation of `PartialEq`, a connection between A to B is the same as
  /// a connection B to A (commutative).
  all_parallel_routes: HashMap<Route, ParallelRoutes>,
}

impl Map {
  /// Generates a `Map`, encapsulating all parallel routes in the game.
  fn new() -> Self {
    Self {
      all_parallel_routes: HashMap::from([
        // Atlanta.
        (
          Route(City::Atlanta, City::Charleston),
          parallel_routes! {2, None},
        ),
        (
          Route(City::Atlanta, City::Miami),
          parallel_routes! {5, Some(Blue)},
        ),
        (
          Route(City::Atlanta, City::Nashville),
          parallel_routes! {1, None},
        ),
        (
          Route(City::Atlanta, City::NewOrleans),
          parallel_routes! {5, Some(Orange), Some(Yellow)},
        ),
        (
          Route(City::Atlanta, City::Raleigh),
          parallel_routes! {2, None, None},
        ),
        // Boston.
        (
          Route(City::Boston, City::Montreal),
          parallel_routes! {2, None, None},
        ),
        (
          Route(City::Boston, City::NewYork),
          parallel_routes! {2, Some(Yellow), Some(Red)},
        ),
        // Calgary.
        (
          Route(City::Calgary, City::Helena),
          parallel_routes! {4, None},
        ),
        (
          Route(City::Calgary, City::Seattle),
          parallel_routes! {4, None},
        ),
        (
          Route(City::Calgary, City::Vancouver),
          parallel_routes! {3, None},
        ),
        (
          Route(City::Calgary, City::Winnipeg),
          parallel_routes! {6, Some(White)},
        ),
        // Charleston.
        (
          Route(City::Charleston, City::Miami),
          parallel_routes! {4, Some(Pink)},
        ),
        (
          Route(City::Charleston, City::Raleigh),
          parallel_routes! {2, None},
        ),
        // Chicago.
        (
          Route(City::Chicago, City::Duluth),
          parallel_routes! {3, Some(Red)},
        ),
        (
          Route(City::Chicago, City::Omaha),
          parallel_routes! {4, Some(Blue)},
        ),
        (
          Route(City::Chicago, City::Pittsburgh),
          parallel_routes! {3, Some(Black), Some(Orange)},
        ),
        (
          Route(City::Chicago, City::SaintLouis),
          parallel_routes! {2, Some(Green), Some(White)},
        ),
        (
          Route(City::Chicago, City::Toronto),
          parallel_routes! {4, Some(White)},
        ),
        // Dallas.
        (
          Route(City::Dallas, City::ElPaso),
          parallel_routes! {4, Some(Red)},
        ),
        (
          Route(City::Dallas, City::Houston),
          parallel_routes! {1, None, None},
        ),
        (
          Route(City::Dallas, City::LittleRock),
          parallel_routes! {2, None},
        ),
        (
          Route(City::Dallas, City::OklahomaCity),
          parallel_routes! {2, None, None},
        ),
        // Denver.
        (
          Route(City::Denver, City::Helena),
          parallel_routes! {4, Some(Green)},
        ),
        (
          Route(City::Denver, City::KansasCity),
          parallel_routes! {4, Some(Black), Some(Orange)},
        ),
        (
          Route(City::Denver, City::OklahomaCity),
          parallel_routes! {4, Some(Red)},
        ),
        (
          Route(City::Denver, City::Omaha),
          parallel_routes! {4, Some(Pink)},
        ),
        (
          Route(City::Denver, City::Phoenix),
          parallel_routes! {5, Some(White)},
        ),
        (
          Route(City::Denver, City::SaltLakeCity),
          parallel_routes! {3, Some(Red), Some(Yellow)},
        ),
        (
          Route(City::Denver, City::SantaFe),
          parallel_routes! {2, None},
        ),
        // Duluth.
        (
          Route(City::Duluth, City::Helena),
          parallel_routes! {6, Some(Orange)},
        ),
        (
          Route(City::Duluth, City::Omaha),
          parallel_routes! {2, None, None},
        ),
        (
          Route(City::Duluth, City::SaultStMarie),
          parallel_routes! {3, None},
        ),
        (
          Route(City::Duluth, City::Toronto),
          parallel_routes! {6, Some(Pink)},
        ),
        (
          Route(City::Duluth, City::Winnipeg),
          parallel_routes! {4, Some(Black)},
        ),
        // El Paso.
        (
          Route(City::ElPaso, City::Houston),
          parallel_routes! {6, Some(Green)},
        ),
        (
          Route(City::ElPaso, City::LosAngeles),
          parallel_routes! {6, Some(Black)},
        ),
        (
          Route(City::ElPaso, City::OklahomaCity),
          parallel_routes! {5, Some(Yellow)},
        ),
        (
          Route(City::ElPaso, City::Phoenix),
          parallel_routes! {3, None},
        ),
        (
          Route(City::ElPaso, City::SantaFe),
          parallel_routes! {2, None},
        ),
        // Helena.
        (
          Route(City::Helena, City::Omaha),
          parallel_routes! {5, Some(Red)},
        ),
        (
          Route(City::Helena, City::SaltLakeCity),
          parallel_routes! {3, Some(Pink)},
        ),
        (
          Route(City::Helena, City::Seattle),
          parallel_routes! {6, Some(Yellow)},
        ),
        (
          Route(City::Helena, City::Winnipeg),
          parallel_routes! {4, Some(Blue)},
        ),
        // Houston.
        (
          Route(City::Houston, City::NewOrleans),
          parallel_routes! {2, None},
        ),
        // Kansas City.
        (
          Route(City::KansasCity, City::SaintLouis),
          parallel_routes! {2, Some(Blue), Some(Pink)},
        ),
        (
          Route(City::KansasCity, City::OklahomaCity),
          parallel_routes! {2, None, None},
        ),
        (
          Route(City::KansasCity, City::Omaha),
          parallel_routes! {1, None, None},
        ),
        // Las Vegas.
        (
          Route(City::LasVegas, City::LosAngeles),
          parallel_routes! {2, None},
        ),
        (
          Route(City::LasVegas, City::SaltLakeCity),
          parallel_routes! {3, Some(Orange)},
        ),
        // Little Rock.
        (
          Route(City::LittleRock, City::Nashville),
          parallel_routes! {3, Some(White)},
        ),
        (
          Route(City::LittleRock, City::NewOrleans),
          parallel_routes! {3, None},
        ),
        (
          Route(City::LittleRock, City::OklahomaCity),
          parallel_routes! {2, None},
        ),
        (
          Route(City::LittleRock, City::SaintLouis),
          parallel_routes! {2, None},
        ),
        // Los Angeles.
        (
          Route(City::LosAngeles, City::Phoenix),
          parallel_routes! {3, None},
        ),
        (
          Route(City::LosAngeles, City::SanFrancisco),
          parallel_routes! {3, Some(Pink), Some(Yellow)},
        ),
        // Miami.
        (
          Route(City::Miami, City::NewOrleans),
          parallel_routes! {6, Some(Red)},
        ),
        // Montreal.
        (
          Route(City::Montreal, City::NewYork),
          parallel_routes! {3, Some(Blue)},
        ),
        (
          Route(City::Montreal, City::SaultStMarie),
          parallel_routes! {5, Some(Black)},
        ),
        (
          Route(City::Montreal, City::Toronto),
          parallel_routes! {3, None},
        ),
        // Nashville.
        (
          Route(City::Nashville, City::Pittsburgh),
          parallel_routes! {4, Some(Yellow)},
        ),
        (
          Route(City::Nashville, City::Raleigh),
          parallel_routes! {3, Some(Black)},
        ),
        (
          Route(City::Nashville, City::SaintLouis),
          parallel_routes! {2, None},
        ),
        // New York.
        (
          Route(City::NewYork, City::Pittsburgh),
          parallel_routes! {2, Some(Green), Some(White)},
        ),
        (
          Route(City::NewYork, City::Washington),
          parallel_routes! {2, Some(Red), Some(Yellow)},
        ),
        // Oklahoma City.
        (
          Route(City::OklahomaCity, City::SantaFe),
          parallel_routes! {3, Some(Blue)},
        ),
        // Phoenix.
        (
          Route(City::Phoenix, City::SantaFe),
          parallel_routes! {3, None},
        ),
        // Pittsburgh.
        (
          Route(City::Pittsburgh, City::Raleigh),
          parallel_routes! {2, None},
        ),
        (
          Route(City::Pittsburgh, City::SaintLouis),
          parallel_routes! {5, Some(Green)},
        ),
        (
          Route(City::Pittsburgh, City::Toronto),
          parallel_routes! {2, None},
        ),
        (
          Route(City::Pittsburgh, City::Washington),
          parallel_routes! {2, None},
        ),
        // Portland.
        (
          Route(City::Portland, City::SaltLakeCity),
          parallel_routes! {6, Some(Blue)},
        ),
        (
          Route(City::Portland, City::SanFrancisco),
          parallel_routes! {5, Some(Green), Some(Pink)},
        ),
        // Raleigh.
        (
          Route(City::Raleigh, City::Washington),
          parallel_routes! {2, None, None},
        ),
        // Salt Lake City.
        (
          Route(City::SaltLakeCity, City::SanFrancisco),
          parallel_routes! {5, Some(Orange), Some(White)},
        ),
        // Sault St. Marie.
        (
          Route(City::SaultStMarie, City::Toronto),
          parallel_routes! {2, None},
        ),
        (
          Route(City::SaultStMarie, City::Winnipeg),
          parallel_routes! {6, None},
        ),
        // Seattle.
        (
          Route(City::Seattle, City::Portland),
          parallel_routes! {1, None, None},
        ),
        (
          Route(City::Seattle, City::Vancouver),
          parallel_routes! {1, None, None},
        ),
      ]),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parallel_routes_macro_with_one_empty_color() {
    assert_eq!(
      parallel_routes! {2, None},
      ParallelRoutes {
        parallel_routes: smallvec![ParallelRoute::new(None)],
        length: 2
      }
    );
  }

  #[test]
  fn parallel_routes_macro_with_two_empty_colors() {
    assert_eq!(
      parallel_routes! {5, None, None},
      ParallelRoutes {
        parallel_routes: smallvec![ParallelRoute::new(None), ParallelRoute::new(None)],
        length: 5
      }
    );
  }

  #[test]
  fn parallel_routes_macro_with_two_colors() {
    assert_eq!(
      parallel_routes! {5, Some(Blue), Some(Orange)},
      ParallelRoutes {
        parallel_routes: smallvec![
          ParallelRoute::new(Some(Blue)),
          ParallelRoute::new(Some(Orange))
        ],
        length: 5
      }
    );
  }

  #[test]
  fn route_opposite_are_equal() {
    assert_eq!(
      Route(City::Atlanta, City::Boston),
      Route(City::Boston, City::Atlanta)
    );
  }

  #[test]
  fn city_tuple_into_route() {
    assert_eq!(
      Route(City::Atlanta, City::Helena),
      (City::Atlanta, City::Helena).into()
    );
  }

  #[test]
  fn route_from_city_tuple() {
    assert_eq!(
      Route::from((City::Atlanta, City::Helena)),
      Route(City::Atlanta, City::Helena)
    );
  }
}
