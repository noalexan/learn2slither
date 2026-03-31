use rust_a_nibbler_wrapper::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
};

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum Action {
    Forward,
    Left,
    Right,
}

impl Action {
    pub fn into_direction(self, d: Direction) -> Direction {
        match self {
            Action::Left => match d {
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
            },
            Action::Forward => d,
            Action::Right => match d {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
            },
        }
    }
}

impl Distribution<Action> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        use rand::seq::IteratorRandom;
        Action::iter().choose(rng).unwrap()
    }
}
