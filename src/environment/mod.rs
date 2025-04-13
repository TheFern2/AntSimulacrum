use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, CircleShape, Color, Transformable, Shape};
use sfml::system::Vector2f;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::pheromone::PheromoneSystem;
use crate::colony::Colony;
use crate::ant::Ant;

// Cell size in pixels
const CELL_SIZE: f32 = 10.0;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Wall,
    Food,
    AntNest,
}

pub struct Environment {
    width: u32,
    height: u32,
    grid_width: usize,
    grid_height: usize,
    grid: Vec<CellType>,
    food_amounts: HashMap<(usize, usize), f32>,
    pheromone_system: PheromoneSystem,
    colonies: Vec<Colony>,
}

impl Environment {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let grid_width = (window_width as f32 / CELL_SIZE) as usize;
        let grid_height = (window_height as f32 / CELL_SIZE) as usize;
        
        Self {
            width: window_width,
            height: window_height,
            grid_width,
            grid_height,
            grid: vec![CellType::Empty; grid_width * grid_height],
            food_amounts: HashMap::new(),
            pheromone_system: PheromoneSystem::new(window_width, window_height, CELL_SIZE),
            colonies: Vec::new(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update pheromones
        self.pheromone_system.update(delta_time);
        
        // Update colonies one at a time to avoid borrowing issues
        let colony_count = self.colonies.len();
        for i in 0..colony_count {
            // Take out the colony temporarily to avoid borrowing the entire self
            let mut colony = std::mem::take(&mut self.colonies[i]);
            
            // Update the colony
            colony.update(delta_time, self);
            
            // Put the colony back
            self.colonies[i] = colony;
        }
        
        // Update food regeneration, environmental effects, etc.
        // For now, we'll keep this simple
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        // Render grid
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let cell_type = self.get_cell(x, y);
                
                match cell_type {
                    CellType::Empty => {},
                    CellType::Wall => {
                        let mut wall = RectangleShape::new();
                        wall.set_size(Vector2f::new(CELL_SIZE, CELL_SIZE));
                        wall.set_position(Vector2f::new(x as f32 * CELL_SIZE, y as f32 * CELL_SIZE));
                        wall.set_fill_color(Color::rgb(100, 80, 60)); // Brown
                        window.draw(&wall);
                    },
                    CellType::Food => {
                        let mut food = CircleShape::new(CELL_SIZE / 2.0, 6);
                        food.set_position(Vector2f::new(x as f32 * CELL_SIZE, y as f32 * CELL_SIZE));
                        food.set_fill_color(Color::rgb(50, 200, 50)); // Green
                        window.draw(&food);
                    },
                    CellType::AntNest => {
                        let mut nest = CircleShape::new(CELL_SIZE * 2.0, 32);
                        nest.set_position(Vector2f::new((x as f32 - 1.5) * CELL_SIZE, (y as f32 - 1.5) * CELL_SIZE));
                        nest.set_fill_color(Color::rgb(150, 100, 50)); // Brown
                        window.draw(&nest);
                    },
                }
            }
        }
        
        // Render pheromones
        self.pheromone_system.render(window);
        
        // Render colonies and ants
        for colony in &self.colonies {
            colony.render(window);
        }
    }
    
    pub fn add_wall(&mut self, x: f32, y: f32) {
        let (grid_x, grid_y) = self.screen_to_grid(x, y);
        if self.is_valid_position(grid_x, grid_y) {
            self.set_cell(grid_x, grid_y, CellType::Wall);
        }
    }
    
    pub fn add_food(&mut self, x: f32, y: f32) {
        let (grid_x, grid_y) = self.screen_to_grid(x, y);
        if self.is_valid_position(grid_x, grid_y) {
            self.set_cell(grid_x, grid_y, CellType::Food);
            self.food_amounts.insert((grid_x, grid_y), 100.0); // Start with 100 units of food
        }
    }
    
    pub fn add_ant_nest(&mut self, x: f32, y: f32) {
        let (grid_x, grid_y) = self.screen_to_grid(x, y);
        // Make sure we have enough space for the nest (it's larger than a single cell)
        if self.is_valid_position(grid_x, grid_y) {
            // Create a 3x3 area for the nest
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = grid_x as isize + dx;
                    let ny = grid_y as isize + dy;
                    if nx >= 0 && nx < self.grid_width as isize && 
                       ny >= 0 && ny < self.grid_height as isize {
                        self.set_cell(nx as usize, ny as usize, CellType::AntNest);
                    }
                }
            }
            
            // Calculate the center position of the nest in screen coordinates
            let colony_x = (grid_x as f32 + 0.5) * CELL_SIZE;
            let colony_y = (grid_y as f32 + 0.5) * CELL_SIZE;
            
            println!("DEBUG: Creating new colony at screen=({:.1},{:.1}), grid=({},{}), colony_pos=({:.1},{:.1})", 
                x, y, grid_x, grid_y, colony_x, colony_y);
                
            // Create a new colony at this location with the correct radius matching the ant food delivery distance check
            let colony_position = Vector2f::new(colony_x, colony_y);
            let colony_radius = 30.0; // Match the distance check in the ant delivery code
            self.colonies.push(Colony::new(colony_position, colony_radius));
        }
    }
    
    pub fn remove_object(&mut self, x: f32, y: f32) {
        let (grid_x, grid_y) = self.screen_to_grid(x, y);
        if self.is_valid_position(grid_x, grid_y) {
            self.set_cell(grid_x, grid_y, CellType::Empty);
            self.food_amounts.remove(&(grid_x, grid_y));
        }
    }
    
    // Convert screen coordinates to grid coordinates
    pub fn screen_to_grid(&self, x: f32, y: f32) -> (usize, usize) {
        let grid_x = (x / CELL_SIZE) as usize;
        let grid_y = (y / CELL_SIZE) as usize;
        (grid_x, grid_y)
    }
    
    // Get cell type at grid coordinates
    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if self.is_valid_position(x, y) {
            self.grid[y * self.grid_width + x]
        } else {
            CellType::Empty
        }
    }
    
    // Set cell type at grid coordinates
    fn set_cell(&mut self, x: usize, y: usize, cell_type: CellType) {
        if self.is_valid_position(x, y) {
            self.grid[y * self.grid_width + x] = cell_type;
        }
    }
    
    // Check if grid coordinates are valid
    fn is_valid_position(&self, x: usize, y: usize) -> bool {
        x < self.grid_width && y < self.grid_height
    }
    
    // Get the pheromone system with mutable reference
    pub fn pheromone_system(&mut self) -> &mut PheromoneSystem {
        &mut self.pheromone_system
    }
    
    // Get the pheromone system with immutable reference
    pub fn pheromone_system_ref(&self) -> &PheromoneSystem {
        &self.pheromone_system
    }
    
    // Get mutable reference to colonies
    pub fn get_colonies(&mut self) -> &mut Vec<Colony> {
        &mut self.colonies
    }
    
    // Get the environment width
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    // Get the environment height
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    // Resize the environment's grid to match the new window size
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let new_grid_width = (new_width as f32 / CELL_SIZE) as usize;
        let new_grid_height = (new_height as f32 / CELL_SIZE) as usize;
        
        // If the new size is larger, expand the grid
        if new_grid_width > self.grid_width || new_grid_height > self.grid_height {
            let mut new_grid = vec![CellType::Empty; new_grid_width * new_grid_height];
            
            // Copy existing grid data to the new grid
            for y in 0..self.grid_height {
                for x in 0..self.grid_width {
                    if x < new_grid_width && y < new_grid_height {
                        new_grid[y * new_grid_width + x] = self.grid[y * self.grid_width + x];
                    }
                }
            }
            
            self.grid = new_grid;
            self.grid_width = new_grid_width;
            self.grid_height = new_grid_height;
        }
        
        // Update internal width and height
        self.width = new_width;
        self.height = new_height;
        
        // Resize the pheromone system
        self.pheromone_system = PheromoneSystem::new(new_width, new_height, CELL_SIZE);
        // Note: This will reset pheromones, but it's simpler than implementing
        // a resize method for the pheromone system
    }
    
    pub fn get_grid_width(&self) -> usize {
        self.grid_width
    }
    
    pub fn get_grid_height(&self) -> usize {
        self.grid_height
    }
    
    pub fn get_grid(&self) -> &Vec<CellType> {
        &self.grid
    }
    
    pub fn set_grid(&mut self, grid: Vec<CellType>, width: usize, height: usize) {
        self.grid = grid;
        self.grid_width = width;
        self.grid_height = height;
    }
    
    pub fn get_food_amounts(&self) -> &HashMap<(usize, usize), f32> {
        &self.food_amounts
    }
    
    pub fn set_food_amounts(&mut self, food_amounts: HashMap<(usize, usize), f32>) {
        self.food_amounts = food_amounts;
    }
    
    pub fn get_all_colonies(&self) -> &Vec<Colony> {
        &self.colonies
    }
    
    pub fn clear_colonies(&mut self) {
        self.colonies.clear();
    }
    
    pub fn add_colony(&mut self, colony: Colony) {
        self.colonies.push(colony);
    }
} 