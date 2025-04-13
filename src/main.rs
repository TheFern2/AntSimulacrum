extern crate sfml;
extern crate rand;
extern crate serde;
extern crate serde_json;

mod ant;
mod colony;
mod environment;
mod game;
mod pheromone;
mod ui;
mod ecs;
mod save;

use game::Game;

fn main() {
    let mut game = Game::new(800, 600, "Ant Simulacrum");
    game.run();
}