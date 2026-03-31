use crate::environment::Environment;
use crate::q_table::{GetBest, GetRow, Loadable, QTable, Savable, q};
use clap::Parser;
use minifb::{Window, WindowOptions};
use rust_a_nibbler_wrapper::*;
use state::State;
use std::collections::HashMap;

pub mod action;
pub mod environment;
pub mod q_table;
pub mod state;

const LEARNING_RATE: f64 = 0.2;
const GAMMA: f64 = 0.9;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

const SIMULTANEOUS: usize = 1024;
const GENERATIONS: usize = 10;

const ADDED_MOVES_ON_GROW: usize = 32;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, conflicts_with = "demo", default_value_t = GENERATIONS)]
    generations: usize,

    #[arg(short, long, conflicts_with = "demo", default_value_t = SIMULTANEOUS)]
    populations: usize,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(long)]
    load: Option<String>,

    #[arg(long, conflicts_with = "demo")]
    save: Option<String>,

    #[arg(long, requires = "load", default_value_t = false)]
    demo: bool,

    #[arg(long, conflicts_with = "demo", default_value_t = false)]
    headless: bool,

    #[arg(long, default_value_t = 10)]
    width: usize,

    #[arg(long, default_value_t = 10)]
    height: usize,
}

fn main() {
    let args = Args::parse();

    let mut q_table: QTable = match &args.load {
        Some(path) => QTable::load(path).unwrap(),
        _ => HashMap::new(),
    };

    let (mut window, mut framebuffer) = if args.headless {
        (None, None)
    } else {
        (
            Window::new(
                "qLearning - nibbler",
                WIDTH,
                HEIGHT,
                WindowOptions::default(),
            )
            .ok(),
            Some(Box::new([0xff0000; WIDTH * HEIGHT])),
        )
    };

    if args.demo {
        loop {
            let mut board = Board::new(args.width, args.height);
            let mut remaining_moves = 64;

            if !window.as_ref().unwrap().is_open() {
                break;
            }

            loop {
                if !window.as_ref().unwrap().is_open() || board.is_stopped() || remaining_moves <= 0
                {
                    break;
                }

                let state = State::from_board(&board);
                let action = q_table.get(&state).unwrap().best();

                let mut snake = board.snake_mut();
                let length = snake.length();
                let dir = snake.direction();

                snake.change_direction(action.into_direction(dir));
                board.update();

                if length < board.snake().length() {
                    remaining_moves += 32;
                }

                remaining_moves -= 1;

                if let Some(fb) = framebuffer.as_mut() {
                    for y in 0..HEIGHT {
                        for x in 0..WIDTH {
                            let tile_x =
                                (x as f64 / WIDTH as f64 * board.width() as f64).floor() as usize;
                            let tile_y =
                                (y as f64 / HEIGHT as f64 * board.height() as f64).floor() as usize;

                            fb[(y) * WIDTH + (x)] = match board.at(tile_x, tile_y) {
                                TileType::Wall => 0xe6e6e6,
                                TileType::Empty => 0x1a1a1a,
                                TileType::GreenApple => 0x33cc33,
                                TileType::RedApple => 0xcc3333,
                                TileType::SnakeBody => 0xcc804d,
                                TileType::Way => 0x1a2633,
                            }
                        }
                    }

                    if let Some(w) = window.as_mut() {
                        w.update_with_buffer(fb.as_ref(), WIDTH, HEIGHT).unwrap();
                    }
                }
            }

            if args.verbose {
                let score = board.score();
                let length = board.snake().length();

                println!(
                    "\x1b[31;1m()\x1b[0m => \x1b[32;1mscord: {:5}\x1b[0m, \x1b[33;1mlength: {:3}\x1b[0m",
                    score, length
                );
            }
        }
    } else {
        for generation in 0..args.generations {
            let mut environments: Vec<Environment> = (0..args.populations)
                .map(|population| Environment::new(generation, population))
                .collect();

            loop {
                let active_environments: Vec<_> = environments
                    .iter_mut()
                    .filter(|env| !env.board.is_stopped() && env.remaining_moves > 0)
                    .collect();

                if window.as_ref().is_some_and(|window| !window.is_open())
                    || active_environments.is_empty()
                {
                    break;
                }

                for env in active_environments {
                    let state = env.get_state();
                    let action = env.get_action(&mut q_table);

                    let mut snake = env.board.snake_mut();
                    let snake_length = snake.length();
                    let dir = snake.direction();

                    let q_value = *q(&mut q_table, state, action);

                    snake.change_direction(action.into_direction(dir));
                    env.board.update();

                    let reward: f64 = if snake_length < env.board.snake().length() {
                        env.remaining_moves += ADDED_MOVES_ON_GROW;
                        20.0
                    } else if snake_length > env.board.snake().length() {
                        -5.0
                    } else if env.board.snake().is_dead() {
                        -10.0
                    } else {
                        -0.025
                    } + (snake_length as f64
                        / ((env.board.width() - 2) * (env.board.height() - 2)) as f64
                        * 5.0);

                    let ns = env.get_state();
                    let na = q_table.row(ns).best();
                    let nq = *q(&mut q_table, ns, na);

                    let updated_q_value = q_value + LEARNING_RATE * (reward + GAMMA * nq - q_value);
                    *q(&mut q_table, state, action) = updated_q_value;

                    env.remaining_moves -= 1;

                    if !args.headless {
                        let boards_on_row = args.populations.isqrt();

                        let board_width = WIDTH as f64 / boards_on_row as f64;
                        let board_height = HEIGHT as f64 / boards_on_row as f64;

                        let board_x =
                            ((env.population % boards_on_row) as f64 * board_width) as usize;
                        let board_y =
                            ((env.population / boards_on_row) as f64 * board_height) as usize;

                        let cell_width = board_width / env.board.width() as f64;
                        let cell_height = board_height / env.board.height() as f64;

                        for y in 0..(board_height.floor() as usize) {
                            for x in 0..(board_width.floor() as usize) {
                                let tile_x = (x as f64 / cell_width).floor() as usize;
                                let tile_y = (y as f64 / cell_height).floor() as usize;

                                if let Some(fb) = framebuffer.as_mut() {
                                    fb[(y + board_y) * WIDTH + (x + board_x)] =
                                        match env.board.at(tile_x, tile_y) {
                                            TileType::Wall => 0xe6e6e6,
                                            TileType::Empty => 0x1a1a1a,
                                            TileType::GreenApple => 0x33cc33,
                                            TileType::RedApple => 0xcc3333,
                                            TileType::SnakeBody => 0xcc804d,
                                            TileType::Way => 0x1a2633,
                                        }
                                }
                            }
                        }
                    }
                }

                if let Some(w) = window.as_mut()
                    && let Some(fb) = framebuffer.as_ref()
                {
                    w.update_with_buffer(fb.as_ref(), WIDTH, HEIGHT).unwrap();
                }
            }

            if args.verbose {
                let scores = environments.iter().map(|env| env.board.score());
                let len = scores.len();
                let min = scores.clone().min().unwrap_or_default();
                let max = scores.clone().max().unwrap_or_default();
                let sum: usize = scores.sum();
                let len_max = environments
                    .iter()
                    .map(|env| env.board.snake().length())
                    .max()
                    .unwrap_or_default();

                println!(
                    "\x1b[31;1mGeneration {:5}/{}\x1b[0m => \x1b[32;1mmin: {:5}\x1b[0m, \x1b[33;1mmax: {:5}\x1b[0m, \x1b[34;1mavrg: {:5}\x1b[0m, \x1b[35;1mlen (max): {:2}\x1b[0m",
                    generation + 1,
                    args.generations,
                    min,
                    max,
                    sum / len,
                    len_max
                );
            }
        }

        if let Some(path) = &args.save {
            q_table.save(&path).unwrap()
        };
    }
}
