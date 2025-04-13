extern crate sfml;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

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
    // Initialize logger with timestamp and debug level
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .with_utc_timestamps()
        .init()
        .unwrap();
    
    log::info!("Starting Ant Simulacrum");
    
    // Create a new game instance and run it
    let mut game = Game::new(1200, 800, "Ant Simulacrum");
    game.run();
}