extern crate sfml;
extern crate rand;

mod ant;
mod colony;
mod environment;
mod game;
mod pheromone;
mod ui;

use game::Game;

fn main() {
    let mut game = Game::new(800, 600, "Ant Simulacrum");
    game.run();
}