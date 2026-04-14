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

        let tile_or_wall = |x: Option<usize>, y: Option<usize>| match (x, y) {
            (Some(x), Some(y)) if x < board.width() && y < board.height() => board.at(x, y),
            _ => TileType::Wall,
        };

        State {
            up: core::array::from_fn(|i| tile_or_wall(Some(head.x), head.y.checked_sub(i))),
            down: core::array::from_fn(|i| tile_or_wall(Some(head.x), head.y.checked_add(i))),
            left: core::array::from_fn(|i| tile_or_wall(head.x.checked_sub(i), Some(head.y))),
            right: core::array::from_fn(|i| tile_or_wall(head.x.checked_add(i), Some(head.y))),
        }
    }
}
