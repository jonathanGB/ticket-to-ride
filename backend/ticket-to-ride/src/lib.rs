//! # The Ticket To Ride library!
//!
//! This crate contains all the modules to create a *Ticket To Ride* game,
//! managing map and player interactions, and running the game.

#![feature(test)]
extern crate test;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate lazy_static;

/// Module that defines the various types of cards ([`card::TrainColor`] and [`card::DestinationCard`]),
/// and the [`card::CardDealer`] in charge of interacting with the decks of cards.
pub mod card;

/// Simple module that defines all the [`city::City`] variants, and connections between them
/// as [`city::CityToCity`] tuples.
pub mod city;
pub mod game_phase;
pub mod game_state;

/// Module that mostly pertains to the [`map::Map`], its routes -- and who claims them.
pub mod map;

/// Modules that defines what a [`player::Player`] is, what actions they can take, and whether
/// they are allowed to fulfill them.
pub mod player;
