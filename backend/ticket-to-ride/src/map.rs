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
    /// The color of this specific route.
    /// The `Wild` color means that any color matches.
    train_color: TrainColor,
}

impl ParallelRoute {
    /// Returns a `ParalleRoute` with the given color.
    /// By default, a route is not claimed.
    fn new(train_color: TrainColor) -> Self {
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
    /// Depending on the number of players (>3), parallel routes might be claimed simultaneously.
    /// In all cases, parallel routes cannot be claimed by the same player.
    parallel_routes_allowed: bool,
}

impl Map {
    /// Generates a `Map`, encapsulating all parallel routes in the game.
    ///
    /// Depending on the number of players (>3), parallel routes might be claimed simultaneously.
    /// In all cases, parallel routes cannot be claimed by the same player.
    pub fn new(num_players: usize) -> Self {
        Self {
            // Parallel routes can be claimed iff there is more than three players.
            parallel_routes_allowed: num_players > 3,
            all_parallel_routes: HashMap::from([
                // Atlanta.
                (
                    Route(City::Atlanta, City::Charleston),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::Atlanta, City::Miami),
                    parallel_routes! {5, Blue},
                ),
                (
                    Route(City::Atlanta, City::Nashville),
                    parallel_routes! {1, Wild},
                ),
                (
                    Route(City::Atlanta, City::NewOrleans),
                    parallel_routes! {5, Orange, Yellow},
                ),
                (
                    Route(City::Atlanta, City::Raleigh),
                    parallel_routes! {2, Wild, Wild},
                ),
                // Boston.
                (
                    Route(City::Boston, City::Montreal),
                    parallel_routes! {2, Wild, Wild},
                ),
                (
                    Route(City::Boston, City::NewYork),
                    parallel_routes! {2, Yellow, Red},
                ),
                // Calgary.
                (
                    Route(City::Calgary, City::Helena),
                    parallel_routes! {4, Wild},
                ),
                (
                    Route(City::Calgary, City::Seattle),
                    parallel_routes! {4, Wild},
                ),
                (
                    Route(City::Calgary, City::Vancouver),
                    parallel_routes! {3, Wild},
                ),
                (
                    Route(City::Calgary, City::Winnipeg),
                    parallel_routes! {6, White},
                ),
                // Charleston.
                (
                    Route(City::Charleston, City::Miami),
                    parallel_routes! {4, Pink},
                ),
                (
                    Route(City::Charleston, City::Raleigh),
                    parallel_routes! {2, Wild},
                ),
                // Chicago.
                (
                    Route(City::Chicago, City::Duluth),
                    parallel_routes! {3, Red},
                ),
                (
                    Route(City::Chicago, City::Omaha),
                    parallel_routes! {4, Blue},
                ),
                (
                    Route(City::Chicago, City::Pittsburgh),
                    parallel_routes! {3, Black, Orange},
                ),
                (
                    Route(City::Chicago, City::SaintLouis),
                    parallel_routes! {2, Green, White},
                ),
                (
                    Route(City::Chicago, City::Toronto),
                    parallel_routes! {4, White},
                ),
                // Dallas.
                (Route(City::Dallas, City::ElPaso), parallel_routes! {4, Red}),
                (
                    Route(City::Dallas, City::Houston),
                    parallel_routes! {1, Wild, Wild},
                ),
                (
                    Route(City::Dallas, City::LittleRock),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::Dallas, City::OklahomaCity),
                    parallel_routes! {2, Wild, Wild},
                ),
                // Denver.
                (
                    Route(City::Denver, City::Helena),
                    parallel_routes! {4, Green},
                ),
                (
                    Route(City::Denver, City::KansasCity),
                    parallel_routes! {4, Black, Orange},
                ),
                (
                    Route(City::Denver, City::OklahomaCity),
                    parallel_routes! {4, Red},
                ),
                (Route(City::Denver, City::Omaha), parallel_routes! {4, Pink}),
                (
                    Route(City::Denver, City::Phoenix),
                    parallel_routes! {5, White},
                ),
                (
                    Route(City::Denver, City::SaltLakeCity),
                    parallel_routes! {3, Red, Yellow},
                ),
                (
                    Route(City::Denver, City::SantaFe),
                    parallel_routes! {2, Wild},
                ),
                // Duluth.
                (
                    Route(City::Duluth, City::Helena),
                    parallel_routes! {6, Orange},
                ),
                (
                    Route(City::Duluth, City::Omaha),
                    parallel_routes! {2, Wild, Wild},
                ),
                (
                    Route(City::Duluth, City::SaultStMarie),
                    parallel_routes! {3, Wild},
                ),
                (
                    Route(City::Duluth, City::Toronto),
                    parallel_routes! {6, Pink},
                ),
                (
                    Route(City::Duluth, City::Winnipeg),
                    parallel_routes! {4, Black},
                ),
                // El Paso.
                (
                    Route(City::ElPaso, City::Houston),
                    parallel_routes! {6, Green},
                ),
                (
                    Route(City::ElPaso, City::LosAngeles),
                    parallel_routes! {6, Black},
                ),
                (
                    Route(City::ElPaso, City::OklahomaCity),
                    parallel_routes! {5, Yellow},
                ),
                (
                    Route(City::ElPaso, City::Phoenix),
                    parallel_routes! {3, Wild},
                ),
                (
                    Route(City::ElPaso, City::SantaFe),
                    parallel_routes! {2, Wild},
                ),
                // Helena.
                (Route(City::Helena, City::Omaha), parallel_routes! {5, Red}),
                (
                    Route(City::Helena, City::SaltLakeCity),
                    parallel_routes! {3, Pink},
                ),
                (
                    Route(City::Helena, City::Seattle),
                    parallel_routes! {6, Yellow},
                ),
                (
                    Route(City::Helena, City::Winnipeg),
                    parallel_routes! {4, Blue},
                ),
                // Houston.
                (
                    Route(City::Houston, City::NewOrleans),
                    parallel_routes! {2, Wild},
                ),
                // Kansas City.
                (
                    Route(City::KansasCity, City::SaintLouis),
                    parallel_routes! {2, Blue, Pink},
                ),
                (
                    Route(City::KansasCity, City::OklahomaCity),
                    parallel_routes! {2, Wild, Wild},
                ),
                (
                    Route(City::KansasCity, City::Omaha),
                    parallel_routes! {1, Wild, Wild},
                ),
                // Las Vegas.
                (
                    Route(City::LasVegas, City::LosAngeles),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::LasVegas, City::SaltLakeCity),
                    parallel_routes! {3, Orange},
                ),
                // Little Rock.
                (
                    Route(City::LittleRock, City::Nashville),
                    parallel_routes! {3, White},
                ),
                (
                    Route(City::LittleRock, City::NewOrleans),
                    parallel_routes! {3, Wild},
                ),
                (
                    Route(City::LittleRock, City::OklahomaCity),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::LittleRock, City::SaintLouis),
                    parallel_routes! {2, Wild},
                ),
                // Los Angeles.
                (
                    Route(City::LosAngeles, City::Phoenix),
                    parallel_routes! {3, Wild},
                ),
                (
                    Route(City::LosAngeles, City::SanFrancisco),
                    parallel_routes! {3, Pink, Yellow},
                ),
                // Miami.
                (
                    Route(City::Miami, City::NewOrleans),
                    parallel_routes! {6, Red},
                ),
                // Montreal.
                (
                    Route(City::Montreal, City::NewYork),
                    parallel_routes! {3, Blue},
                ),
                (
                    Route(City::Montreal, City::SaultStMarie),
                    parallel_routes! {5, Black},
                ),
                (
                    Route(City::Montreal, City::Toronto),
                    parallel_routes! {3, Wild},
                ),
                // Nashville.
                (
                    Route(City::Nashville, City::Pittsburgh),
                    parallel_routes! {4, Yellow},
                ),
                (
                    Route(City::Nashville, City::Raleigh),
                    parallel_routes! {3, Black},
                ),
                (
                    Route(City::Nashville, City::SaintLouis),
                    parallel_routes! {2, Wild},
                ),
                // New York.
                (
                    Route(City::NewYork, City::Pittsburgh),
                    parallel_routes! {2, Green, White},
                ),
                (
                    Route(City::NewYork, City::Washington),
                    parallel_routes! {2, Red, Yellow},
                ),
                // Oklahoma City.
                (
                    Route(City::OklahomaCity, City::SantaFe),
                    parallel_routes! {3, Blue},
                ),
                // Phoenix.
                (
                    Route(City::Phoenix, City::SantaFe),
                    parallel_routes! {3, Wild},
                ),
                // Pittsburgh.
                (
                    Route(City::Pittsburgh, City::Raleigh),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::Pittsburgh, City::SaintLouis),
                    parallel_routes! {5, Green},
                ),
                (
                    Route(City::Pittsburgh, City::Toronto),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::Pittsburgh, City::Washington),
                    parallel_routes! {2, Wild},
                ),
                // Portland.
                (
                    Route(City::Portland, City::SaltLakeCity),
                    parallel_routes! {6, Blue},
                ),
                (
                    Route(City::Portland, City::SanFrancisco),
                    parallel_routes! {5, Green, Pink},
                ),
                // Raleigh.
                (
                    Route(City::Raleigh, City::Washington),
                    parallel_routes! {2, Wild, Wild},
                ),
                // Salt Lake City.
                (
                    Route(City::SaltLakeCity, City::SanFrancisco),
                    parallel_routes! {5, Orange, White},
                ),
                // Sault St. Marie.
                (
                    Route(City::SaultStMarie, City::Toronto),
                    parallel_routes! {2, Wild},
                ),
                (
                    Route(City::SaultStMarie, City::Winnipeg),
                    parallel_routes! {6, Wild},
                ),
                // Seattle.
                (
                    Route(City::Seattle, City::Portland),
                    parallel_routes! {1, Wild, Wild},
                ),
                (
                    Route(City::Seattle, City::Vancouver),
                    parallel_routes! {1, Wild, Wild},
                ),
            ]),
        }
    }

    /// Request from a player `player_id` to claim a specific parallel route between two cities.
    ///
    /// A multitude of verifications are applied to make sure that the player has the right to claim this route.
    ///
    /// If any verification fails, we return the error message.
    /// Otherwise, we mutate the map to mark the parallel route as claimed, and return the length of the route claimed.
    pub fn claim_parallel_route(
        &mut self,
        route: Route,
        parallel_route_index: usize,
        cards: &Vec<TrainColor>,
        player_id: usize,
    ) -> Result<u8, String> {
        let curr_routes = self.all_parallel_routes.get_mut(&route);
        if curr_routes.is_none() {
            return Err(format!(
                "No routes exist between {} and {}.",
                route.0, route.1
            ));
        }

        let curr_routes = curr_routes.unwrap();

        if parallel_route_index >= curr_routes.parallel_routes.len() {
            return Err(format!(
                "The selected route ({}) between {} and {} does not exist.",
                parallel_route_index, route.0, route.1
            ));
        }

        if curr_routes.length != cards.len() as u8 {
            return Err(format!(
                "A route between {} and {} needs {} cards, but {} were provided.",
                route.0,
                route.1,
                curr_routes.length,
                cards.len()
            ));
        }

        let num_parallel_routes = curr_routes.parallel_routes.len();
        if num_parallel_routes > 1 {
            let other_parallel_route =
                &curr_routes.parallel_routes[parallel_route_index + 1 % num_parallel_routes];

            match other_parallel_route.claimed_by {
                Some(claimer) if claimer == player_id => {
                    return Err(format!(
                        "Cannot claim more than one route between {} and {}.",
                        route.0, route.1
                    ));
                }
                Some(_) if !self.parallel_routes_allowed => {
                    return Err(format!(
                        "A route is already claimed by someone else between {} and {}.",
                        route.0, route.1
                    ));
                }
                _ => {}
            }
        }

        let claimed_route = &mut curr_routes.parallel_routes[parallel_route_index];
        if claimed_route.claimed_by.is_some() {
            return Err(format!(
                "The selected route between {} and {} is already claimed.",
                route.0, route.1
            ));
        }

        // Amongst the cards used to claim this route, we want to know what is their shared color.
        // All cards used should have the same color, ignoring wild cards.
        // In case all cards used are wild cards, the common color is "wild".
        let mut common_color = Wild;
        for card in cards {
            if card.is_wild() {
                continue;
            }

            if common_color.is_wild() {
                common_color = *card;
                continue;
            }

            if common_color != *card {
                return Err(format!(
                    "Cannot claim a route with {} and {} cards.",
                    common_color, card
                ));
            }
        }

        if common_color.is_not_wild()
            && claimed_route.train_color.is_not_wild()
            && claimed_route.train_color != common_color
        {
            return Err(format!(
                "Cannot claim a {} route with {} cards.",
                claimed_route.train_color, common_color
            ));
        }

        // Due diligence is done, the player can rightfully claim the route.
        claimed_route.claimed_by = Some(player_id);
        Ok(curr_routes.length)
    }

    fn is_destination_connected(&self, route: Route, claimed_routes: &Vec<Route>) -> bool {
        unimplemented!()
    }

    fn get_longest_route(&self, claimed_routes: &Vec<Route>) -> u16 {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_routes_macro_with_one_empty_color() {
        assert_eq!(
            parallel_routes! {2, Wild},
            ParallelRoutes {
                parallel_routes: smallvec![ParallelRoute::new(Wild)],
                length: 2
            }
        );
    }

    #[test]
    fn parallel_routes_macro_with_two_empty_colors() {
        assert_eq!(
            parallel_routes! {5, Wild, Wild},
            ParallelRoutes {
                parallel_routes: smallvec![ParallelRoute::new(Wild), ParallelRoute::new(Wild)],
                length: 5
            }
        );
    }

    #[test]
    fn parallel_routes_macro_with_two_colors() {
        assert_eq!(
            parallel_routes! {5, Blue, Orange},
            ParallelRoutes {
                parallel_routes: smallvec![ParallelRoute::new(Blue), ParallelRoute::new(Orange)],
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
