use serde::{Serialize, Deserialize};
use sfml::system::Vector2f;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::collections::HashMap;

use crate::environment::{Environment, CellType};
use crate::colony::Colony;
use crate::ant::Ant;
use crate::pheromone::PheromoneType;
use crate::game::Game;

// Serializable versions of our game structs
#[derive(Serialize, Deserialize, Clone)]
pub struct SavedVector2f {
    x: f32,
    y: f32,
}

impl From<Vector2f> for SavedVector2f {
    fn from(vec: Vector2f) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
        }
    }
}

impl Into<Vector2f> for SavedVector2f {
    fn into(self) -> Vector2f {
        Vector2f::new(self.x, self.y)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedAnt {
    pub position: SavedVector2f,
    pub direction: f32,
    pub speed: f32,
    pub carrying_food: bool,
    pub home_position: SavedVector2f,
    pub pheromone_deposit_timer: f32,
}

impl SavedAnt {
    pub fn from_ant(ant: &Ant) -> Self {
        Self {
            position: SavedVector2f::from(ant.get_position()),
            direction: ant.get_direction(),
            speed: ant.get_speed(),
            carrying_food: ant.is_carrying_food(),
            home_position: SavedVector2f::from(ant.get_home_position()),
            pheromone_deposit_timer: ant.get_pheromone_deposit_timer(),
        }
    }
    
    pub fn to_ant(&self) -> Ant {
        let mut ant = Ant::new(self.position.x, self.position.y);
        ant.set_direction(self.direction);
        ant.set_speed(self.speed);
        ant.set_carrying_food(self.carrying_food);
        ant.set_home_position(Vector2f::new(self.home_position.x, self.home_position.y));
        ant.set_pheromone_deposit_timer(self.pheromone_deposit_timer);
        ant
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedColony {
    position: SavedVector2f,
    ants: Vec<SavedAnt>,
    food_stored: f32,
    max_ants: usize,
    food_deliveries: u32,
}

impl SavedColony {
    pub fn from_colony(colony: &Colony) -> Self {
        let ants: Vec<SavedAnt> = colony.get_ants().iter()
            .map(|ant| SavedAnt::from_ant(ant))
            .collect();
            
        Self {
            position: SavedVector2f::from(colony.get_position()),
            ants,
            food_stored: colony.get_food_stored(),
            max_ants: colony.get_max_ants(),
            food_deliveries: colony.get_food_deliveries(),
        }
    }
    
    pub fn to_colony(&self) -> Colony {
        let mut colony = Colony::new(Vector2f::new(self.position.x, self.position.y), 30.0);
        
        // Remove default ants and replace with saved ants
        colony.clear_ants();
        
        for saved_ant in &self.ants {
            colony.add_ant(saved_ant.to_ant());
        }
        
        colony.set_food_stored(self.food_stored);
        colony.set_max_ants(self.max_ants);
        colony.set_food_deliveries(self.food_deliveries);
        
        colony
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedPheromone {
    grid_x: usize,
    grid_y: usize,
    pheromone_type: PheromoneType,
    strength: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SavedEnvironment {
    width: u32,
    height: u32,
    grid_width: usize,
    grid_height: usize,
    grid: Vec<CellType>,
    food_amounts: Vec<((usize, usize), f32)>,
    pheromones: Vec<SavedPheromone>,
    colonies: Vec<SavedColony>,
}

impl SavedEnvironment {
    pub fn from_environment(env: &Environment) -> Self {
        let pheromones = env.pheromone_system_ref().get_all_pheromones().iter()
            .map(|((grid_x, grid_y, ptype), strength)| {
                SavedPheromone {
                    grid_x: *grid_x,
                    grid_y: *grid_y,
                    pheromone_type: *ptype,
                    strength: *strength,
                }
            })
            .collect();
            
        let colonies = env.get_all_colonies().iter()
            .map(|colony| SavedColony::from_colony(colony))
            .collect();
            
        Self {
            width: env.get_width(),
            height: env.get_height(),
            grid_width: env.get_grid_width(),
            grid_height: env.get_grid_height(),
            grid: env.get_grid().clone(),
            food_amounts: env.get_food_amounts().clone().into_iter().collect(),
            pheromones,
            colonies,
        }
    }
    
    pub fn to_environment(&self) -> Environment {
        let mut env = Environment::new(self.width, self.height);
        
        // Set the grid
        env.set_grid(self.grid.clone(), self.grid_width, self.grid_height);
        
        // Set food amounts
        let food_amounts: HashMap<(usize, usize), f32> = self.food_amounts.clone().into_iter().collect();
        env.set_food_amounts(food_amounts);
        
        // Add pheromones
        for pheromone in &self.pheromones {
            env.pheromone_system().add_pheromone_at_grid(
                pheromone.grid_x,
                pheromone.grid_y,
                pheromone.pheromone_type,
                pheromone.strength
            );
        }
        
        // Add colonies
        env.clear_colonies();
        for colony in &self.colonies {
            env.add_colony(colony.to_colony());
        }
        
        env
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedGame {
    pub environment: SavedEnvironment,
    pub interaction_mode: String,
    pub simulation_speed: f32,
    pub paused: bool,
    pub test_ants: Vec<SavedAnt>,
}

impl SavedGame {
    pub fn from_game(game: &Game) -> Self {
        Self {
            environment: SavedEnvironment::from_environment(game.get_environment()),
            interaction_mode: format!("{:?}", game.get_interaction_mode()),
            simulation_speed: game.get_simulation_speed(),
            paused: game.is_paused(),
            test_ants: game.get_test_ants().iter()
                .map(|ant| SavedAnt::from_ant(ant))
                .collect(),
        }
    }
}

pub fn save_game_state(path: &Path, game: &Game) -> io::Result<()> {
    let game_state = SavedGame::from_game(game);
    let serialized = serde_json::to_string_pretty(&game_state)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn load_game_state(path: &Path) -> io::Result<SavedGame> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let game_state: SavedGame = serde_json::from_str(&contents)?;
    Ok(game_state)
} 