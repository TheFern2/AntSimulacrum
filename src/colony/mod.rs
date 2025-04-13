use sfml::graphics::RenderWindow;
use sfml::system::Vector2f;

use crate::ant::Ant;
use crate::environment::Environment;

#[derive(Default)]
pub struct Colony {
    position: Vector2f,
    ants: Vec<Ant>,
    food_stored: f32,
    max_ants: usize,
}

impl Colony {
    pub fn new(x: f32, y: f32) -> Self {
        let mut colony = Self {
            position: Vector2f::new(x, y),
            ants: Vec::new(),
            food_stored: 0.0,
            max_ants: 50,  // Start with a small cap
        };
        
        // Create initial ants
        for _ in 0..10 {
            colony.ants.push(Ant::new(x, y));
        }
        
        colony
    }
    
    pub fn update(&mut self, delta_time: f32, environment: &mut Environment) {
        // Update all ants
        for ant in &mut self.ants {
            ant.update(delta_time, environment);
        }
        
        // Colony management (spawn new ants, etc.)
        // For now, just maintain a certain number of ants
        if self.ants.len() < self.max_ants && self.food_stored > 10.0 {
            self.ants.push(Ant::new(self.position.x, self.position.y));
            self.food_stored -= 10.0;
        }
    }
    
    pub fn add_food(&mut self, amount: f32) {
        self.food_stored += amount;
        
        // Food allows colony to grow
        if self.food_stored > 100.0 && self.max_ants < 200 {
            self.max_ants += 1;
        }
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        // Render all ants
        for ant in &self.ants {
            ant.render(window);
        }
    }
    
    pub fn get_statistics(&self) -> (usize, f32, usize) {
        (self.ants.len(), self.food_stored, self.max_ants)
    }
} 