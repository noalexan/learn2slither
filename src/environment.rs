use rand::{Rng, rngs::ThreadRng};
use rust_a_nibbler_wrapper::*;

use crate::action::Action;
use crate::q_table::{QTable, GetRow, GetBest};
use crate::state::State;

const EPSILON: f64 = 0.2;
const MOVES: usize = 128;

#[derive(Debug)]
pub struct Environment {
    pub generation: usize,
    pub population: usize,
    rng: ThreadRng,
    pub board: Board,
    pub remaining_moves: usize,
}

impl Environment {
    pub fn new(generation: usize, population: usize) -> Self {
        Self {
            generation,
            population,
            rng: rand::rng(),
            board: Board::new(12, 12),
            remaining_moves: MOVES,
        }
    }

    pub fn get_state(&self) -> State {
        State::from_board(&self.board)
    }

    pub fn get_action(&mut self, q_table: &mut QTable) -> Action {
        if self.rng.random::<f64>() < EPSILON * (0.995f64).powi(self.generation as i32 + 1) {
            self.rng.random()
        } else {
            q_table.row(self.get_state()).best()
        }
    }
}
