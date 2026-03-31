use rust_a_nibbler_wrapper::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct State {
    up: [TileType; 3],
    down: [TileType; 3],
    left: [TileType; 3],
    right: [TileType; 3],
}

impl State {
    pub fn from_board(board: &Board) -> Self {
        let snake = board.snake();
        let head = snake.head();

        State {
            up: core::array::from_fn(|i| {
                let y = head.y - i;
                if y < board.height() {
                    board.at(head.x, y)
                } else {
                    TileType::Wall
                }
            }),
            down: core::array::from_fn(|i| {
                let y = head.y + i;
                if y < board.height() {
                    board.at(head.x, y)
                } else {
                    TileType::Wall
                }
            }),
            left: core::array::from_fn(|i| {
                let x = head.x - i;
                if x < board.width() {
                    board.at(x, head.y)
                } else {
                    TileType::Wall
                }
            }),
            right: core::array::from_fn(|i| {
                let x = head.x + i;
                if x < board.width() {
                    board.at(x, head.y)
                } else {
                    TileType::Wall
                }
            }),
        }
    }
}
