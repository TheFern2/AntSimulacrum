use sfml::graphics::{RenderWindow, RenderTarget, CircleShape, Color, Transformable, BlendMode, RenderStates, Shape};
use sfml::system::Vector2f;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,
    Home,
}

pub struct PheromoneSystem {
    // Grid-based pheromone storage
    // Key: (grid_x, grid_y, pheromone_type)
    // Value: strength (0.0 to 1.0)
    pheromones: HashMap<(usize, usize, PheromoneType), f32>,
    grid_size: f32,
    width: usize,
    height: usize,
}

impl PheromoneSystem {
    pub fn new(width: u32, height: u32, grid_size: f32) -> Self {
        let width_cells = (width as f32 / grid_size) as usize;
        let height_cells = (height as f32 / grid_size) as usize;
        
        Self {
            pheromones: HashMap::new(),
            grid_size,
            width: width_cells,
            height: height_cells,
        }
    }
    
    pub fn add_pheromone(&mut self, x: f32, y: f32, pheromone_type: PheromoneType, strength: f32) {
        let grid_x = (x / self.grid_size) as usize;
        let grid_y = (y / self.grid_size) as usize;
        
        if grid_x >= self.width || grid_y >= self.height {
            return;
        }
        
        let key = (grid_x, grid_y, pheromone_type);
        let current_strength = self.pheromones.get(&key).unwrap_or(&0.0);
        
        // Pheromones add up to a maximum
        let new_strength = (current_strength + strength).min(1.0);
        self.pheromones.insert(key, new_strength);
    }
    
    pub fn get_pheromone(&self, x: f32, y: f32, pheromone_type: &PheromoneType) -> f32 {
        let grid_x = (x / self.grid_size) as usize;
        let grid_y = (y / self.grid_size) as usize;
        
        if grid_x >= self.width || grid_y >= self.height {
            return 0.0;
        }
        
        *self.pheromones.get(&(grid_x, grid_y, pheromone_type.clone())).unwrap_or(&0.0)
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Evaporation rate controls how quickly pheromones fade
        // Current rate: 0.04 * delta_time (4% per second if delta_time is in seconds)
        // Decrease this value for longer-lasting pheromones that create more persistent trails
        let evaporation_rate = 0.005 * delta_time; // Reduced for much longer-lasting pheromones
        
        let mut to_remove = Vec::new();
        
        for (key, strength) in self.pheromones.iter_mut() {
            // Reduce strength of each pheromone by the evaporation rate
            *strength -= evaporation_rate;
            
            // Threshold for removing weak pheromones (currently 0.003)
            // Decrease this value to allow weaker pheromones to persist longer
            if *strength <= 0.001 { // Lower threshold for pheromone removal
                to_remove.push(*key);
            }
        }
        
        // Remove all pheromones that fell below the threshold
        for key in to_remove {
            self.pheromones.remove(&key);
        }
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        for ((grid_x, grid_y, pheromone_type), strength) in &self.pheromones {
            let x = *grid_x as f32 * self.grid_size;
            let y = *grid_y as f32 * self.grid_size;
            
            let mut pheromone = CircleShape::new(self.grid_size / 2.0, 8);
            pheromone.set_position(Vector2f::new(x, y));
            
            // Enhance alpha value to make pheromones more visible
            let alpha = (*strength * 255.0).min(255.0) as u8;
            
            match pheromone_type {
                PheromoneType::Food => {
                    // Brighter green for food pheromones
                    pheromone.set_fill_color(Color::rgba(0, 255, 100, alpha));
                }
                PheromoneType::Home => {
                    // Brighter magenta for home pheromones
                    pheromone.set_fill_color(Color::rgba(255, 50, 255, alpha));
                }
            }
            
            let mut states = RenderStates::default();
            states.blend_mode = BlendMode::ADD;
            window.draw_with_renderstates(&pheromone, &states);
        }
    }
    
    pub fn get_all_pheromones(&self) -> &HashMap<(usize, usize, PheromoneType), f32> {
        &self.pheromones
    }
    
    pub fn add_pheromone_at_grid(&mut self, grid_x: usize, grid_y: usize, pheromone_type: PheromoneType, strength: f32) {
        if grid_x >= self.width || grid_y >= self.height {
            return;
        }
        
        let key = (grid_x, grid_y, pheromone_type);
        let current_strength = self.pheromones.get(&key).unwrap_or(&0.0);
        
        // Pheromones add up to a maximum
        let new_strength = (current_strength + strength).min(1.0);
        self.pheromones.insert(key, new_strength);
    }
} 