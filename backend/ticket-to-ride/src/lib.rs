//! # The Ticket To Ride library!
//!
//! This crate contains all the modules to create a *Ticket To Ride* game,
//! managig map and player interactions, and running the game.

#![feature(test)]
extern crate test;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate lazy_static;

/// Module that defines the various types of cards (train and destination cards),
/// and the [`card::CardDealer`] in charge of interacting with the decks of cards.
pub mod card;
pub mod city;
pub mod game_phase;
pub mod game_state;
pub mod map;
pub mod player;
