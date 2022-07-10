use crate::card::TrainColor;
use crate::card::TrainColor::*;
use crate::city::{City, CityToCity};

use array_init::array_init;
use smallvec::SmallVec;
use std::cell::Cell;
use std::cmp::max;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use threadpool::ThreadPool;

lazy_static! {
    static ref THREAD_POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::default());
}

// This assumes that the largest city, as ordered in `City`, is Winnipeg.
const NUM_CITIES: usize = City::Winnipeg as usize;

// Helena has the highest number of neighbors, which is 7 adjacent cities.
const MAX_ROUTES_PER_CITY: usize = 7;

/// There can be multiple "parallel" routes between two cities.
/// `Route` represents one of them.
#[derive(Clone, Debug, PartialEq)]
struct Route {
    /// By whom this route is claimed, if any.
    /// Because the overall map holds a separate route from A to B and from B to A, we encapsulate the
    /// owner inside of an `Rc<Cell<...>>` for shared interior mutability of the claimer.
    claimer: Rc<Cell<Option<usize>>>,
    /// The color of this specific route.
    /// The `Wild` color means that any color matches.
    train_color: TrainColor,
    /// The distance between two cities. This is analogous to the number of train cards needed to claim the route.
    length: u8,
}

impl Route {
    /// Returns a `Route` with the given color and length.
    /// By default, a route is not claimed.
    fn new(train_color: TrainColor, length: u8) -> Self {
        Self {
            claimer: Rc::new(Cell::new(None)),
            train_color,
            length,
        }
    }

    /// The player ID claiming this route, if any.
    fn claimer(&self) -> Option<usize> {
        self.claimer.get()
    }

    fn set_claimer(&self, player_id: usize) {
        self.claimer.set(Some(player_id));
    }
}

/// All routes connecting two adjacent cities.
/// There is up to two "parallel" routes between two cities.
type ParallelRoutes = SmallVec<[Route; 2]>;

/// Holds the parallel routes connecting City A to City B,
/// and the same routes in the opposite direction, i.e. from City B to City A.
type BidirectionalCityRouteMapping = [(CityToCity, ParallelRoutes); 2];

// All city-route mappings contained in the US map.
type UsMap = [BidirectionalCityRouteMapping; 78];

/// Convenience macro to generate "parallel" routes between two cities.
macro_rules! parallel_routes {
  ($l:literal, $($train_colors:expr),+) => ({
    let parallel_routes: ParallelRoutes = smallvec![$(Route::new($train_colors, $l)),+];
    parallel_routes
    })
}

/// Holds the information about a route successfully claimed by a player.
#[derive(Debug, PartialEq)]
pub struct ClaimedRoute {
    pub route: CityToCity,
    pub parallel_route_index: usize,
    pub length: u8,
}

/// The authoritative state of the map, per game.
/// This can be mutated as players claim routes throughout the game.
/// Not thread-safe!
pub struct Map {
    /// Maps the concept of two cities being adjacent to the underlying parallel routes between the two.
    all_parallel_routes: BTreeMap<CityToCity, ParallelRoutes>,
    /// Depending on the number of players (>3), parallel routes might be claimed simultaneously.
    /// In all cases, parallel routes cannot be claimed by the same player.
    parallel_routes_allowed: bool,
}

impl Map {
    fn get_range_of_routes_starting_at_city(city: City) -> RangeInclusive<CityToCity> {
        // This assumes that the smallest city, as ordered in `City`, is Atlanta,
        // and the largest city is Winnipeg.
        (city, City::Atlanta)..=(city, City::Winnipeg)
    }

    fn build_bidirectional_city_route_mapping(
        (start, end): CityToCity,
        parallel_routes: ParallelRoutes,
    ) -> BidirectionalCityRouteMapping {
        [
            ((start, end), parallel_routes.clone()),
            ((end, start), parallel_routes),
        ]
    }

    fn build_us_map() -> UsMap {
        [
            // Atlanta.
            Self::build_bidirectional_city_route_mapping(
                (City::Atlanta, City::Charleston),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Atlanta, City::Miami),
                parallel_routes! {5, Blue},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Atlanta, City::Nashville),
                parallel_routes! {1, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Atlanta, City::NewOrleans),
                parallel_routes! {5, Orange, Yellow},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Atlanta, City::Raleigh),
                parallel_routes! {2, Wild, Wild},
            ),
            // Boston.
            Self::build_bidirectional_city_route_mapping(
                (City::Boston, City::Montreal),
                parallel_routes! {2, Wild, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Boston, City::NewYork),
                parallel_routes! {2, Yellow, Red},
            ),
            // Calgary.
            Self::build_bidirectional_city_route_mapping(
                (City::Calgary, City::Helena),
                parallel_routes! {4, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Calgary, City::Seattle),
                parallel_routes! {4, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Calgary, City::Vancouver),
                parallel_routes! {3, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Calgary, City::Winnipeg),
                parallel_routes! {6, White},
            ),
            // Charleston.
            Self::build_bidirectional_city_route_mapping(
                (City::Charleston, City::Miami),
                parallel_routes! {4, Pink},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Charleston, City::Raleigh),
                parallel_routes! {2, Wild},
            ),
            // Chicago.
            Self::build_bidirectional_city_route_mapping(
                (City::Chicago, City::Duluth),
                parallel_routes! {3, Red},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Chicago, City::Omaha),
                parallel_routes! {4, Blue},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Chicago, City::Pittsburgh),
                parallel_routes! {3, Black, Orange},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Chicago, City::SaintLouis),
                parallel_routes! {2, Green, White},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Chicago, City::Toronto),
                parallel_routes! {4, White},
            ),
            // Dallas.
            Self::build_bidirectional_city_route_mapping(
                (City::Dallas, City::ElPaso),
                parallel_routes! {4, Red},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Dallas, City::Houston),
                parallel_routes! {1, Wild, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Dallas, City::LittleRock),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Dallas, City::OklahomaCity),
                parallel_routes! {2, Wild, Wild},
            ),
            // Denver.
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::Helena),
                parallel_routes! {4, Green},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::KansasCity),
                parallel_routes! {4, Black, Orange},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::OklahomaCity),
                parallel_routes! {4, Red},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::Omaha),
                parallel_routes! {4, Pink},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::Phoenix),
                parallel_routes! {5, White},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::SaltLakeCity),
                parallel_routes! {3, Red, Yellow},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Denver, City::SantaFe),
                parallel_routes! {2, Wild},
            ),
            // Duluth.
            Self::build_bidirectional_city_route_mapping(
                (City::Duluth, City::Helena),
                parallel_routes! {6, Orange},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Duluth, City::Omaha),
                parallel_routes! {2, Wild, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Duluth, City::SaultStMarie),
                parallel_routes! {3, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Duluth, City::Toronto),
                parallel_routes! {6, Pink},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Duluth, City::Winnipeg),
                parallel_routes! {4, Black},
            ),
            // El Paso.
            Self::build_bidirectional_city_route_mapping(
                (City::ElPaso, City::Houston),
                parallel_routes! {6, Green},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::ElPaso, City::LosAngeles),
                parallel_routes! {6, Black},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::ElPaso, City::OklahomaCity),
                parallel_routes! {5, Yellow},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::ElPaso, City::Phoenix),
                parallel_routes! {3, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::ElPaso, City::SantaFe),
                parallel_routes! {2, Wild},
            ),
            // Helena.
            Self::build_bidirectional_city_route_mapping(
                (City::Helena, City::Omaha),
                parallel_routes! {5, Red},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Helena, City::SaltLakeCity),
                parallel_routes! {3, Pink},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Helena, City::Seattle),
                parallel_routes! {6, Yellow},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Helena, City::Winnipeg),
                parallel_routes! {4, Blue},
            ),
            // Houston.
            Self::build_bidirectional_city_route_mapping(
                (City::Houston, City::NewOrleans),
                parallel_routes! {2, Wild},
            ),
            // Kansas City.
            Self::build_bidirectional_city_route_mapping(
                (City::KansasCity, City::SaintLouis),
                parallel_routes! {2, Blue, Pink},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::KansasCity, City::OklahomaCity),
                parallel_routes! {2, Wild, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::KansasCity, City::Omaha),
                parallel_routes! {1, Wild, Wild},
            ),
            // Las Vegas.
            Self::build_bidirectional_city_route_mapping(
                (City::LasVegas, City::LosAngeles),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::LasVegas, City::SaltLakeCity),
                parallel_routes! {3, Orange},
            ),
            // Little Rock.
            Self::build_bidirectional_city_route_mapping(
                (City::LittleRock, City::Nashville),
                parallel_routes! {3, White},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::LittleRock, City::NewOrleans),
                parallel_routes! {3, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::LittleRock, City::OklahomaCity),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::LittleRock, City::SaintLouis),
                parallel_routes! {2, Wild},
            ),
            // Los Angeles.
            Self::build_bidirectional_city_route_mapping(
                (City::LosAngeles, City::Phoenix),
                parallel_routes! {3, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::LosAngeles, City::SanFrancisco),
                parallel_routes! {3, Pink, Yellow},
            ),
            // Miami.
            Self::build_bidirectional_city_route_mapping(
                (City::Miami, City::NewOrleans),
                parallel_routes! {6, Red},
            ),
            // Montreal.
            Self::build_bidirectional_city_route_mapping(
                (City::Montreal, City::NewYork),
                parallel_routes! {3, Blue},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Montreal, City::SaultStMarie),
                parallel_routes! {5, Black},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Montreal, City::Toronto),
                parallel_routes! {3, Wild},
            ),
            // Nashville.
            Self::build_bidirectional_city_route_mapping(
                (City::Nashville, City::Pittsburgh),
                parallel_routes! {4, Yellow},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Nashville, City::Raleigh),
                parallel_routes! {3, Black},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Nashville, City::SaintLouis),
                parallel_routes! {2, Wild},
            ),
            // New York.
            Self::build_bidirectional_city_route_mapping(
                (City::NewYork, City::Pittsburgh),
                parallel_routes! {2, Green, White},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::NewYork, City::Washington),
                parallel_routes! {2, Black, Orange},
            ),
            // Oklahoma City.
            Self::build_bidirectional_city_route_mapping(
                (City::OklahomaCity, City::SantaFe),
                parallel_routes! {3, Blue},
            ),
            // Phoenix.
            Self::build_bidirectional_city_route_mapping(
                (City::Phoenix, City::SantaFe),
                parallel_routes! {3, Wild},
            ),
            // Pittsburgh.
            Self::build_bidirectional_city_route_mapping(
                (City::Pittsburgh, City::Raleigh),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Pittsburgh, City::SaintLouis),
                parallel_routes! {5, Green},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Pittsburgh, City::Toronto),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Pittsburgh, City::Washington),
                parallel_routes! {2, Wild},
            ),
            // Portland.
            Self::build_bidirectional_city_route_mapping(
                (City::Portland, City::SaltLakeCity),
                parallel_routes! {6, Blue},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Portland, City::SanFrancisco),
                parallel_routes! {5, Green, Pink},
            ),
            // Raleigh.
            Self::build_bidirectional_city_route_mapping(
                (City::Raleigh, City::Washington),
                parallel_routes! {2, Wild, Wild},
            ),
            // Salt Lake City.
            Self::build_bidirectional_city_route_mapping(
                (City::SaltLakeCity, City::SanFrancisco),
                parallel_routes! {5, Orange, White},
            ),
            // Sault St. Marie.
            Self::build_bidirectional_city_route_mapping(
                (City::SaultStMarie, City::Toronto),
                parallel_routes! {2, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::SaultStMarie, City::Winnipeg),
                parallel_routes! {6, Wild},
            ),
            // Seattle.
            Self::build_bidirectional_city_route_mapping(
                (City::Seattle, City::Portland),
                parallel_routes! {1, Wild, Wild},
            ),
            Self::build_bidirectional_city_route_mapping(
                (City::Seattle, City::Vancouver),
                parallel_routes! {1, Wild, Wild},
            ),
        ]
    }

    /// Generates a `Map`, encapsulating all parallel routes in the game.
    ///
    /// Succeeds if the given number of players is allowed (i.e. must be between two and five, inclusively).
    /// Otherwise, returns an error.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::map::Map;
    ///
    /// let map = Map::new(5);
    /// assert!(map.is_ok());
    ///
    /// let map = Map::new(1);
    /// assert!(map.is_err());
    /// ```
    pub fn new(num_players: usize) -> Result<Self, String> {
        if num_players < 2 || num_players > 5 {
            Err(format!("Cannot create a game with {} players: one must have at least two, and at most 5 players.", num_players))
        } else {
            Ok(Self {
                // Parallel routes can be claimed iff there is more than three players.
                // Otherwise, only one of the routes connecting two cities can be claimed.
                parallel_routes_allowed: num_players > 3,
                all_parallel_routes: BTreeMap::from_iter(
                    Self::build_us_map().into_iter().flatten(),
                ),
            })
        }
    }

    /// Request from a player `player_id` to claim a specific route between two cities.
    ///
    /// As there can be many routes connecting two cities, the request must specify which of the "parallel" routes they want to claim.
    /// As well, the player must provide the cards that are used to claim that route.
    ///
    /// A multitude of verifications are applied to make sure that the player has the right to claim this route.
    /// For instance, a player must use cards of the route's corresponding color in order to claim it.
    ///
    /// If any verification fails, we return the error message.
    /// Otherwise, we mutate the map to mark the parallel route as claimed, and return information about the claimed route.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::city::City;
    /// use ticket_to_ride::map::{ClaimedRoute, Map};
    /// use ticket_to_ride::card::TrainColor;
    ///
    /// let map = Map::new(2).unwrap();
    ///
    /// let route = (City::Raleigh, City::Washington);
    /// let parallel_route_index = 0;
    /// let cards = vec![TrainColor::White, TrainColor::White];
    /// let player_id = 0;
    ///
    /// assert_eq!(
    ///     map.claim_route_for_player(route, parallel_route_index, &cards, player_id),
    ///     Ok(ClaimedRoute{
    ///         route,
    ///         parallel_route_index,
    ///         length: 2
    ///     })
    /// );
    ///
    /// // Same player trying to claim the other parallel route fails.
    /// let parallel_route_index = 1;
    /// assert_eq!(
    ///     map.claim_route_for_player(route, parallel_route_index, &cards, player_id),
    ///     Err(String::from("Cannot claim more than one route between Raleigh and Washington."))
    /// );
    /// ```
    pub fn claim_route_for_player(
        &self,
        route: CityToCity,
        parallel_route_index: usize,
        cards: &Vec<TrainColor>,
        player_id: usize,
    ) -> Result<ClaimedRoute, String> {
        let claimed_route =
            self.can_route_be_claimed_by_player(route, parallel_route_index, cards, player_id)?;

        // Due diligence is done, the player can rightfully claim the route.
        claimed_route.set_claimer(player_id);
        Ok(ClaimedRoute {
            route,
            parallel_route_index,
            length: claimed_route.length,
        })
    }

    fn can_route_be_claimed_by_player(
        &self,
        (start, end): CityToCity,
        parallel_route_index: usize,
        cards: &Vec<TrainColor>,
        player_id: usize,
    ) -> Result<&Route, String> {
        let parallel_routes = self.all_parallel_routes.get(&(start, end));
        if parallel_routes.is_none() {
            return Err(format!("No routes exist between {} and {}.", start, end));
        }

        let parallel_routes = parallel_routes.unwrap();
        if parallel_route_index >= parallel_routes.len() {
            return Err(format!(
                "The selected route ({}) between {} and {} does not exist.",
                parallel_route_index, start, end
            ));
        }

        let claimed_route = &parallel_routes[parallel_route_index];
        if claimed_route.length != cards.len() as u8 {
            return Err(format!(
                "A route between {} and {} needs {} cards, but {} were provided.",
                start,
                end,
                claimed_route.length,
                cards.len()
            ));
        }

        if claimed_route.claimer().is_some() {
            return Err(format!(
                "The selected route between {} and {} is already claimed.",
                start, end
            ));
        }

        let num_parallel_routes = parallel_routes.len();
        if num_parallel_routes > 1 {
            let other_parallel_route =
                &parallel_routes[(parallel_route_index + 1) % num_parallel_routes];
            match other_parallel_route.claimer() {
                Some(claimer) if claimer == player_id => {
                    return Err(format!(
                        "Cannot claim more than one route between {} and {}.",
                        start, end
                    ));
                }
                Some(_) if !self.parallel_routes_allowed => {
                    return Err(format!(
                        "Another route is already claimed by someone else between {} and {}.",
                        start, end
                    ));
                }
                _ => {}
            }
        }

        // Amongst the cards used to claim this route, we want to know what is their color.
        // All cards used to claim a route should have the same color, ignoring wild cards.
        // In case all cards used are wild cards, the common color is "wild".
        let mut common_color = Wild;
        for card in cards {
            if card.is_wild() {
                continue;
            }

            // At this point, the card cannot be a wild card.

            if common_color.is_wild() {
                common_color = *card;
                continue;
            }

            // At this point, we have already iterated through at least one card that is not
            // a wild card. Next cards (including this one) should be the same color.

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
                "Cannot claim a route of color {} with {} cards.",
                claimed_route.train_color, common_color
            ));
        }

        Ok(claimed_route)
    }

    /// Predicate that assess whether a given player has connected two cities on the map, based on their claimed routes.
    ///
    /// Returns true if that is the case, false otherwise.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::city::City;
    /// use ticket_to_ride::map::Map;
    /// use ticket_to_ride::card::TrainColor;
    ///
    /// let map = Map::new(2).unwrap();
    ///
    /// let destination = (City::Raleigh, City::NewYork);
    /// let player_id = 0;
    ///
    /// // Player has not claimed any route.
    /// assert_eq!(map.has_player_fulfilled_destination(destination, player_id), false);
    ///
    /// // Player has partially fulfilled the destination.
    /// let route = (City::Raleigh, City::Washington);
    /// let parallel_route_index = 0;
    /// let cards = vec![TrainColor::White, TrainColor::White];
    /// assert!(map.claim_route_for_player(route, parallel_route_index, &cards, player_id).is_ok());
    /// assert_eq!(map.has_player_fulfilled_destination(destination, player_id), false);
    ///
    /// // Player has fully fulfilled the destination.
    /// let route = (City::Washington, City::NewYork);
    /// let cards = vec![TrainColor::Black, TrainColor::Black];
    /// assert!(map.claim_route_for_player(route, parallel_route_index, &cards, player_id).is_ok());
    /// assert!(map.has_player_fulfilled_destination(destination, player_id));
    /// ```
    pub fn has_player_fulfilled_destination(
        &self,
        (destination_start, destination_end): CityToCity,
        player_id: usize,
    ) -> bool {
        let mut cities_visited = [false; NUM_CITIES];
        let mut cities_to_visit = VecDeque::with_capacity(NUM_CITIES);

        self.extend_neighboring_cities_to_visit_claimed_by_player(
            destination_start,
            player_id,
            &mut cities_visited,
            &mut cities_to_visit,
        );

        while let Some(city) = cities_to_visit.pop_front() {
            if city == destination_end {
                return true;
            }

            self.extend_neighboring_cities_to_visit_claimed_by_player(
                city,
                player_id,
                &mut cities_visited,
                &mut cities_to_visit,
            );
        }

        false
    }

    fn extend_neighboring_cities_to_visit_claimed_by_player(
        &self,
        city: City,
        player_id: usize,
        cities_visited: &mut [bool; NUM_CITIES],
        cities_to_visit: &mut VecDeque<City>,
    ) {
        cities_to_visit.extend(
            self.all_parallel_routes
                .range(Self::get_range_of_routes_starting_at_city(city))
                .filter_map(|((_, end), parallel_routes)| {
                    if cities_visited[*end as usize - 1] {
                        return None;
                    }

                    if parallel_routes
                        .iter()
                        .any(|route| route.claimer() == Some(player_id))
                    {
                        cities_visited[*end as usize - 1] = true;
                        Some(end)
                    } else {
                        None
                    }
                }),
        );
    }

    /// Returns the longest continuous path spanned from the claimed routes.
    ///
    /// Note that a continous path may visit a city multiple times, but may not repeat a path
    /// through a route.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::city::City;
    /// use ticket_to_ride::map::{ClaimedRoute, Map};
    ///
    /// let claimed_routes = vec![
    ///     ClaimedRoute {
    ///         route: (City::ElPaso, City::Phoenix),
    ///         parallel_route_index: 0,
    ///         length: 3,
    ///     },
    ///     ClaimedRoute {
    ///         route: (City::Denver, City::Phoenix),
    ///         parallel_route_index: 0,
    ///         length: 5,
    ///     },
    /// ];
    ///
    /// // Route El Paso -> Phoenix is of length 3.
    /// // Route Phoenix -> Denver is of length 5.
    /// assert_eq!(Map::get_longest_route(&claimed_routes), 8);
    /// ```
    pub fn get_longest_route(claimed_routes: &Vec<ClaimedRoute>) -> u16 {
        let mut cities_to_visit = HashSet::new();
        let mut longest_route = 0;

        // Maps each city to a list of adjacent cities, including the length of the route connecting the two.
        // Start cities are indexed by their usize representation.
        let mut all_routes: [SmallVec<[(City, u8); MAX_ROUTES_PER_CITY]>; NUM_CITIES] =
            array_init(|_| SmallVec::new());

        // Deduplicate the cities that will be explored.
        for claimed_route in claimed_routes {
            let (start, end) = claimed_route.route;

            cities_to_visit.insert(start);
            cities_to_visit.insert(end);

            all_routes[start as usize].push((end, claimed_route.length));
            all_routes[end as usize].push((start, claimed_route.length));
        }

        // Prepare multi-threading.
        let all_routes = Arc::new(all_routes);
        let (tx, rx) = mpsc::sync_channel(0);
        let num_cities_to_visit = cities_to_visit.len();
        let thread_pool = THREAD_POOL.lock().unwrap();

        // Each city will spawn a separate thread from the pool, and compute the longest route
        // starting at that city.
        for city in cities_to_visit {
            let all_routes = all_routes.clone();
            let tx = tx.clone();

            thread_pool.execute(move || {
                tx.send(Self::get_longest_route_from_city(
                    city,
                    &all_routes,
                    HashSet::new(),
                    0,
                ))
                .unwrap();
            });
        }

        for _ in 0..num_cities_to_visit {
            longest_route = max(longest_route, rx.recv().unwrap());
        }

        longest_route
    }

    fn get_longest_route_from_city(
        start: City,
        all_routes: &[SmallVec<[(City, u8); 7]>; NUM_CITIES],
        routes_visited: HashSet<CityToCity>,
        current_length: u16,
    ) -> u16 {
        let mut longest_route_from_city = current_length;

        for (end, length) in &all_routes[start as usize] {
            if routes_visited.contains(&(start, *end)) {
                continue;
            }

            let mut routes_visited = routes_visited.clone();
            routes_visited.insert((start, *end));
            routes_visited.insert((*end, start));

            longest_route_from_city = max(
                longest_route_from_city,
                Self::get_longest_route_from_city(
                    *end,
                    all_routes,
                    routes_visited,
                    current_length + *length as u16,
                ),
            );
        }

        longest_route_from_city
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_routes_macro_with_one_empty_color() {
        let expected_parallel_routes: ParallelRoutes = smallvec![Route::new(Wild, 2)];
        assert_eq!(parallel_routes! {2, Wild}, expected_parallel_routes);
    }

    #[test]
    fn parallel_routes_macro_with_two_empty_colors() {
        let expected_parallel_routes: ParallelRoutes =
            smallvec![Route::new(Wild, 5), Route::new(Wild, 5)];

        assert_eq!(parallel_routes! {5, Wild, Wild}, expected_parallel_routes);
    }

    #[test]
    fn parallel_routes_macro_with_two_colors() {
        let expected_parallel_routes: ParallelRoutes =
            smallvec![Route::new(Blue, 5), Route::new(Orange, 5)];

        assert_eq!(parallel_routes! {5, Blue, Orange}, expected_parallel_routes);
    }

    #[test]
    fn city_range_construction() {
        assert_eq!(
            Map::get_range_of_routes_starting_at_city(City::SanFrancisco),
            (City::SanFrancisco, City::Atlanta)..=(City::SanFrancisco, City::Winnipeg)
        );
    }

    #[test]
    fn get_one_parallel_route_between_adjacent_cities() {
        let map = Map::new(2).unwrap();

        let expected_parallel_routes = parallel_routes! {6, White};
        assert_eq!(
            map.all_parallel_routes
                .get(&(City::Calgary, City::Winnipeg)),
            Some(&expected_parallel_routes)
        );
        assert_eq!(
            map.all_parallel_routes
                .get(&(City::Winnipeg, City::Calgary)),
            Some(&expected_parallel_routes)
        );
    }

    #[test]
    fn get_two_parallel_routes_between_adjacent_cities() {
        let map = Map::new(2).unwrap();

        let expected_parallel_routes = parallel_routes! {2, Blue, Pink};
        assert_eq!(
            map.all_parallel_routes
                .get(&(City::KansasCity, City::SaintLouis)),
            Some(&expected_parallel_routes)
        );
        assert_eq!(
            map.all_parallel_routes
                .get(&(City::SaintLouis, City::KansasCity)),
            Some(&expected_parallel_routes)
        );
    }

    #[test]
    fn get_no_parallel_routes_between_non_adjacent_cities() {
        let map = Map::new(2).unwrap();

        assert_eq!(
            map.all_parallel_routes.get(&(City::Houston, City::NewYork)),
            None
        );
        assert_eq!(
            map.all_parallel_routes.get(&(City::Seattle, City::Miami)),
            None
        );
    }

    #[test]
    fn new_map() {
        for num_players in 0..=7 {
            if num_players < 2 || num_players > 5 {
                assert!(
                    Map::new(num_players).is_err(),
                    "Fails with num_players={num_players}"
                );
            } else {
                assert!(
                    Map::new(num_players).is_ok(),
                    "Fails with num_players={num_players}"
                );
            }
        }
    }

    // Tests for `Map::claim_route_for_player`.

    struct ClaimRouteArgs {
        route: CityToCity,
        parallel_route_index: usize,
        other_parallel_route_index: usize,
        cards: Vec<TrainColor>,
        player_id: usize,
        other_player_id: usize,
    }

    impl Default for ClaimRouteArgs {
        fn default() -> Self {
            Self {
                route: (City::Denver, City::KansasCity),
                parallel_route_index: 1,
                other_parallel_route_index: 0,
                cards: vec![Orange; 4],
                player_id: 0,
                other_player_id: 1,
            }
        }
    }

    #[test]
    fn claim_non_existent_route() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.route = (City::LosAngeles, City::Charleston);

        let expected_result = Err(String::from(
            "No routes exist between Los Angeles and Charleston.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_with_large_route_index() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.parallel_route_index = 10;

        let expected_result = Err(String::from(
            "The selected route (10) between Denver and Kansas City does not exist.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_with_not_enough_cards() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards.clear();

        let expected_result = Err(String::from(
            "A route between Denver and Kansas City needs 4 cards, but 0 were provided.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_with_too_many_cards() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards = vec![Orange; 5];

        let expected_result = Err(String::from(
            "A route between Denver and Kansas City needs 4 cards, but 5 were provided.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_already_owned_by_player() {
        let map = Map::new(2).unwrap();

        let args = ClaimRouteArgs::default();

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        parallel_routes[args.parallel_route_index].set_claimer(args.player_id);

        let expected_result = Err(String::from(
            "The selected route between Denver and Kansas City is already claimed.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_parallel_also_owned_by_player() {
        let map = Map::new(2).unwrap();

        let args = ClaimRouteArgs::default();

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        parallel_routes[args.other_parallel_route_index].set_claimer(args.player_id);

        let expected_result = Err(String::from(
            "Cannot claim more than one route between Denver and Kansas City.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_parallel_route_owned_and_parallel_disabled() {
        // With two players, different players cannot claim parallel routes.
        let map = Map::new(2).unwrap();

        let args = ClaimRouteArgs::default();

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        parallel_routes[args.other_parallel_route_index].set_claimer(args.other_player_id);

        let expected_result = Err(String::from(
            "Another route is already claimed by someone else between Denver and Kansas City.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_parallel_route_owned_but_parallel_enabled() {
        // With four players, different players can claim parallel routes.
        let map = Map::new(4).unwrap();

        let args = ClaimRouteArgs::default();

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        let claimed_route = &parallel_routes[args.parallel_route_index];
        parallel_routes[args.other_parallel_route_index].set_claimer(args.other_player_id);

        assert!(claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(claimed_route.claimer(), Some(args.player_id));
    }

    #[test]
    fn claim_route_for_player_cards_different_colors() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards = vec![Orange, Orange, Blue, Orange];

        let expected_result = Err(String::from(
            "Cannot claim a route with orange and blue cards.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_cards_single_wrong_color() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards = vec![Red; 4];

        let expected_result = Err(String::from(
            "Cannot claim a route of color orange with red cards.",
        ));

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );
    }

    #[test]
    fn claim_route_for_player_cards_single_right_color() {
        let map = Map::new(2).unwrap();

        let args = ClaimRouteArgs::default();
        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        let claimed_route = &parallel_routes[args.parallel_route_index];

        assert!(claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(claimed_route.claimer(), Some(args.player_id));
    }

    #[test]
    fn claim_route_for_player_cards_color_and_wild() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards = vec![Orange, Wild, Wild, Orange];
        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        let claimed_route = &parallel_routes[args.parallel_route_index];

        assert!(claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(claimed_route.claimer(), Some(args.player_id));
    }

    #[test]
    fn claim_route_for_player_cards_only_wild() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.cards = vec![Wild; 4];

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        let claimed_route = &parallel_routes[args.parallel_route_index];

        assert!(claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(claimed_route.claimer(), Some(args.player_id));
    }

    #[test]
    fn claim_wild_route_cards_single_color() {
        let map = Map::new(2).unwrap();

        let mut args = ClaimRouteArgs::default();
        args.route = (City::Pittsburgh, City::Toronto);
        args.parallel_route_index = 0;
        args.cards = vec![Green; 2];

        let parallel_routes = map.all_parallel_routes.get(&args.route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        let claimed_route = &parallel_routes[args.parallel_route_index];

        assert!(claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(claimed_route.claimer(), Some(args.player_id));
    }

    #[test]
    fn claim_route_for_player_impacts_opposite_direction() {
        let map = Map::new(2).unwrap();

        let args = ClaimRouteArgs::default();

        let opposite_direction_parallel_routes =
            map.all_parallel_routes.get(&(args.route.1, args.route.0));
        assert!(opposite_direction_parallel_routes.is_some());
        let opposite_direction_parallel_routes = opposite_direction_parallel_routes.unwrap();
        let opposite_direction_claimed_route =
            &opposite_direction_parallel_routes[args.parallel_route_index];

        assert!(opposite_direction_claimed_route.claimer().is_none());

        let expected_result = Ok(ClaimedRoute {
            route: args.route,
            parallel_route_index: args.parallel_route_index,
            length: opposite_direction_claimed_route.length,
        });

        assert_eq!(
            map.claim_route_for_player(
                args.route,
                args.parallel_route_index,
                &args.cards,
                args.player_id
            ),
            expected_result
        );

        assert_eq!(
            opposite_direction_claimed_route.claimer(),
            Some(args.player_id)
        );
    }

    // Test helper that claims a given route for a given player.
    fn claim_route_for_player(map: &Map, route: &CityToCity, player_id: usize) {
        let parallel_routes = map.all_parallel_routes.get(route);
        assert!(parallel_routes.is_some());
        let parallel_routes = parallel_routes.unwrap();
        parallel_routes[0].set_claimer(player_id);
    }

    // Tests for `Map::has_player_fulfilled_destination`.

    #[test]
    fn destination_not_fulfilled_at_start() {
        let map = Map::new(2).unwrap();

        assert_eq!(
            map.has_player_fulfilled_destination((City::Calgary, City::Winnipeg), 0),
            false
        );
    }

    #[test]
    fn destination_partially_fulfilled() {
        let map = Map::new(2).unwrap();
        let player_id = 0;

        claim_route_for_player(&map, &(City::SaltLakeCity, City::Denver), player_id);

        assert_eq!(
            map.has_player_fulfilled_destination((City::Denver, City::Portland), player_id),
            false
        );
    }

    #[test]
    fn destination_fulfilled_by_another_player() {
        let map = Map::new(2).unwrap();
        let player_id = 0;
        let other_player_id = 1;

        claim_route_for_player(
            &map,
            &(City::SaltLakeCity, City::SanFrancisco),
            other_player_id,
        );
        claim_route_for_player(
            &map,
            &(City::SaltLakeCity, City::SanFrancisco),
            other_player_id,
        );
        claim_route_for_player(&map, &(City::Portland, City::SanFrancisco), other_player_id);

        assert_eq!(
            map.has_player_fulfilled_destination((City::Denver, City::Portland), player_id),
            false
        );
    }

    #[test]
    fn short_destination_fulfilled() {
        let map = Map::new(2).unwrap();
        let player_id = 0;

        claim_route_for_player(&map, &(City::ElPaso, City::Phoenix), player_id);

        assert!(map.has_player_fulfilled_destination((City::Phoenix, City::ElPaso), player_id));
    }

    #[test]
    fn long_destination_fulfilled() {
        // We will claim multiple routes for player 0, and check whether Denver-Portland is fulfilled.
        let map = Map::new(2).unwrap();
        let player_id = 0;

        claim_route_for_player(&map, &(City::SaltLakeCity, City::Denver), player_id);
        claim_route_for_player(&map, &(City::SaltLakeCity, City::SanFrancisco), player_id);
        claim_route_for_player(&map, &(City::Portland, City::SanFrancisco), player_id);
        claim_route_for_player(&map, &(City::SanFrancisco, City::LosAngeles), player_id);
        claim_route_for_player(&map, &(City::Helena, City::SaltLakeCity), player_id);

        assert!(map.has_player_fulfilled_destination((City::Denver, City::Portland), player_id));
    }

    // Tests for `Map::get_longest_route`.

    #[test]
    fn longest_route_zero_length() {
        assert_eq!(Map::get_longest_route(&vec![]), 0);
    }

    #[test]
    fn longest_route_one_length() {
        let claimed_routes = vec![ClaimedRoute {
            route: (City::ElPaso, City::Phoenix),
            parallel_route_index: 0,
            length: 3,
        }];

        // Route El Paso -> Phoenix is of length 3.
        assert_eq!(Map::get_longest_route(&claimed_routes), 3);
    }

    #[test]
    fn longest_route_two_length() {
        let claimed_routes = vec![
            ClaimedRoute {
                route: (City::ElPaso, City::Phoenix),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: (City::Denver, City::Phoenix),
                parallel_route_index: 0,
                length: 5,
            },
        ];

        // Route El Paso -> Phoenix is of length 3.
        // Route Phoenix -> Denver is of length 5.
        assert_eq!(Map::get_longest_route(&claimed_routes), 8);
    }

    #[test]
    fn longest_route_long_line() {
        let claimed_routes = vec![
            ClaimedRoute {
                route: (City::ElPaso, City::Phoenix),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: (City::Denver, City::Phoenix),
                parallel_route_index: 0,
                length: 5,
            },
            ClaimedRoute {
                route: (City::Denver, City::KansasCity),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: (City::KansasCity, City::OklahomaCity),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: (City::OklahomaCity, City::Dallas),
                parallel_route_index: 0,
                length: 2,
            },
        ];

        // Route El Paso -> Phoenix is of length 3.
        // Route Phoenix -> Denver is of length 5.
        // Route Denver -> Kansas City is of length 4.
        // Route Kansas City -> Oklahoma City is of length 2.
        // Route Oklahoma city -> Dallas is of length 2.
        assert_eq!(Map::get_longest_route(&claimed_routes), 16);
    }

    #[test]
    fn longest_route_long_single_loop() {
        let claimed_routes = vec![
            ClaimedRoute {
                route: (City::ElPaso, City::Phoenix),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: (City::Denver, City::Phoenix),
                parallel_route_index: 0,
                length: 5,
            },
            ClaimedRoute {
                route: (City::Denver, City::KansasCity),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: (City::KansasCity, City::OklahomaCity),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: (City::OklahomaCity, City::Dallas),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: (City::Dallas, City::ElPaso),
                parallel_route_index: 0,
                length: 4,
            },
        ];

        // Route El Paso -> Phoenix is of length 3.
        // Route Phoenix -> Denver is of length 5.
        // Route Denver -> Kansas City is of length 4.
        // Route Kansas City -> Oklahoma City is of length 2.
        // Route Oklahoma city -> Dallas is of length 2.
        // Route Dallas -> El Paso is of length 4.
        assert_eq!(Map::get_longest_route(&claimed_routes), 20);
    }

    #[test]
    fn longest_route_realistic() {
        let claimed_routes = vec![
            ClaimedRoute {
                route: ((City::NewOrleans, City::LittleRock)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::LittleRock, City::SaintLouis)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::SaintLouis, City::Chicago)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::Phoenix, City::Denver)),
                parallel_route_index: 0,
                length: 5,
            },
            ClaimedRoute {
                route: ((City::Denver, City::KansasCity)),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: ((City::KansasCity, City::SaintLouis)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::Chicago, City::Toronto)),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: ((City::Toronto, City::Montreal)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Denver, City::SantaFe)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::SantaFe, City::ElPaso)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::SantaFe, City::Phoenix)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Denver, City::OklahomaCity)),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: ((City::OklahomaCity, City::LittleRock)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::NewOrleans, City::Miami)),
                parallel_route_index: 0,
                length: 6,
            },
            ClaimedRoute {
                route: ((City::Vancouver, City::Calgary)),
                parallel_route_index: 0,
                length: 3,
            },
        ];

        // Route Miami -> New Orleans is of length 6.
        // Route New Orleans -> Little Rock is of length 3.
        // Route Little Rock -> Oklahoma City is of length 2.
        // Route Oklahoma City -> Denver is of length 4.
        // Route Denver -> Santa Fe is of length 2.
        // Route Santa Fe -> Phoenix is of length 3.
        // Route Phoenix -> Denver is of length 5.
        // Route Denver -> Kansas City is of length 4.
        // Route Kansas City -> Saint Louis is of length 2.
        // Route Saint Louis -> Chicago is of length 2.
        // Route Chicago -> Toronto is of length 4.
        // Route Toronto -> Montreal is of length 3.
        assert_eq!(Map::get_longest_route(&claimed_routes), 40);
    }

    #[test]
    fn longest_route_convoluted() {
        let claimed_routes = vec![
            ClaimedRoute {
                route: ((City::Portland, City::SaltLakeCity)),
                parallel_route_index: 0,
                length: 6,
            },
            ClaimedRoute {
                route: ((City::SaltLakeCity, City::Helena)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Helena, City::Seattle)),
                parallel_route_index: 0,
                length: 6,
            },
            ClaimedRoute {
                route: ((City::Seattle, City::Portland)),
                parallel_route_index: 0,
                length: 1,
            },
            ClaimedRoute {
                route: ((City::Helena, City::Denver)),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: ((City::Denver, City::SaltLakeCity)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::SaltLakeCity, City::LasVegas)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::LasVegas, City::LosAngeles)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::LosAngeles, City::Phoenix)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Vancouver, City::Calgary)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::OklahomaCity, City::LittleRock)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::NewOrleans, City::Miami)),
                parallel_route_index: 0,
                length: 6,
            },
        ];

        // Route Phoenix -> Los Angeles is of length 3.
        // Route Los Angeles -> Las Vegas is of length 2.
        // Route Las Vegas -> Salt Lake City is of length 3.
        // Route Salt Lake City -> Denver is of length 3.
        // Route Denver -> Helenas is of length 4.
        // Route Helena -> Salt Lake City is of length 3.
        // Route Salt Lake City -> Portland is of length 6.
        // Route Portland -> Seattle is of length 1.
        // Route Seattle -> Helena is of length 6.
        assert_eq!(Map::get_longest_route(&claimed_routes), 31);
    }

    // Micro-benchmarks.
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use test::Bencher;

    #[bench]
    fn benchmark_city_to_city_lookup(b: &mut Bencher) {
        let m = Map::new(2).unwrap();

        let mut routes: Vec<&CityToCity> = m.all_parallel_routes.keys().collect();
        routes.shuffle(&mut thread_rng());

        b.iter(|| {
            for route in routes.iter() {
                test::black_box(m.all_parallel_routes.get(route));
            }
        })
    }

    #[bench]
    fn benchmark_create_us_map(b: &mut Bencher) {
        b.iter(|| Map::new(2).unwrap())
    }

    #[bench]
    fn benchmark_destination_fulfilled(b: &mut Bencher) {
        let map = Map::new(2).unwrap();
        let player_id = 0;

        claim_route_for_player(&map, &(City::SaltLakeCity, City::Denver), player_id);
        claim_route_for_player(&map, &(City::SaltLakeCity, City::Portland), player_id);
        claim_route_for_player(&map, &(City::SaltLakeCity, City::SanFrancisco), player_id);
        claim_route_for_player(&map, &(City::Phoenix, City::Denver), player_id);
        claim_route_for_player(&map, &(City::Phoenix, City::SantaFe), player_id);
        claim_route_for_player(&map, &(City::Denver, City::KansasCity), player_id);
        claim_route_for_player(&map, &(City::Omaha, City::Duluth), player_id);
        claim_route_for_player(&map, &(City::Duluth, City::Winnipeg), player_id);
        claim_route_for_player(&map, &(City::SantaFe, City::ElPaso), player_id);
        claim_route_for_player(&map, &(City::ElPaso, City::Houston), player_id);
        claim_route_for_player(&map, &(City::ElPaso, City::LosAngeles), player_id);
        claim_route_for_player(&map, &(City::SanFrancisco, City::LosAngeles), player_id);
        claim_route_for_player(&map, &(City::Houston, City::NewOrleans), player_id);
        claim_route_for_player(&map, &(City::NewOrleans, City::Miami), player_id);

        b.iter(|| {
            test::black_box(
                map.has_player_fulfilled_destination((City::SantaFe, City::Montreal), player_id),
            )
        });
    }

    #[bench]
    fn benchmark_longest_route(b: &mut Bencher) {
        let claimed_routes = vec![
            ClaimedRoute {
                route: ((City::Portland, City::SaltLakeCity)),
                parallel_route_index: 0,
                length: 6,
            },
            ClaimedRoute {
                route: ((City::SaltLakeCity, City::Helena)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Helena, City::Seattle)),
                parallel_route_index: 0,
                length: 6,
            },
            ClaimedRoute {
                route: ((City::Seattle, City::Portland)),
                parallel_route_index: 0,
                length: 1,
            },
            ClaimedRoute {
                route: ((City::Helena, City::Denver)),
                parallel_route_index: 0,
                length: 4,
            },
            ClaimedRoute {
                route: ((City::Denver, City::SaltLakeCity)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::SaltLakeCity, City::LasVegas)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::LasVegas, City::LosAngeles)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::LosAngeles, City::Phoenix)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::Vancouver, City::Calgary)),
                parallel_route_index: 0,
                length: 3,
            },
            ClaimedRoute {
                route: ((City::OklahomaCity, City::LittleRock)),
                parallel_route_index: 0,
                length: 2,
            },
            ClaimedRoute {
                route: ((City::NewOrleans, City::Miami)),
                parallel_route_index: 0,
                length: 6,
            },
        ];

        b.iter(|| test::black_box(Map::get_longest_route(&claimed_routes)));
    }
}
