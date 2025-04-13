use sfml::graphics::{RenderWindow, Text, Color, Font, Transformable, Shape, RenderTarget};
use sfml::system::Vector2f;
use log::debug;

use crate::ant::Ant;
use crate::environment::Environment;

#[derive(Default)]
pub struct Colony {
    position: Vector2f,
    radius: f32,
    ants: Vec<Ant>,
    food_stored: f32,
    max_ants: usize,
    food_deliveries: u32, // Track the number of food deliveries
}

impl Colony {
    pub fn new(position: Vector2f, radius: f32) -> Self {
        debug!("Creating new colony at position ({},{})", position.x, position.y);
        let mut colony = Self {
            position,
            radius,
            ants: Vec::new(),
            food_stored: 0.0,
            max_ants: 50,  // Start with a small cap
            food_deliveries: 0, // Start with no deliveries
        };
        
        // Create initial ants
        for _ in 0..10 {
            colony.ants.push(Ant::new(position.x, position.y));
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
        self.food_deliveries += 1; // Count each food delivery
        
        println!("DEBUG: Colony at ({:.1},{:.1}) food delivery! Amount: {}, Total stored: {:.1}, Deliveries count: {}", 
            self.position.x, self.position.y, amount, self.food_stored, self.food_deliveries);
        
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
        
        // Render food information above the colony
        self.render_food_info(window);
    }
    
    fn render_food_info(&self, window: &mut RenderWindow) {
        // Create a text representation of food information
        // This depends on having a static font reference, which we can't easily get here
        // We'll use a colored circle with size proportional to food instead
        
        // Draw a circular indicator for stored food - gold/yellow
        let radius = (self.food_stored / 10.0).min(8.0).max(2.0); // Size relative to food amount
        let mut food_indicator = sfml::graphics::CircleShape::new(radius, 6);
        food_indicator.set_fill_color(Color::rgb(255, 215, 0)); // Gold
        food_indicator.set_position(Vector2f::new(
            self.position.x - radius, 
            self.position.y - 30.0 - radius // Position above colony
        ));
        window.draw(&food_indicator);
        
        // Draw a numerical indicator for deliveries - white background with number
        let delivery_radius = 8.0;
        let mut delivery_bg = sfml::graphics::CircleShape::new(delivery_radius, 12);
        delivery_bg.set_fill_color(Color::rgb(255, 255, 255)); // White
        delivery_bg.set_outline_thickness(1.0);
        delivery_bg.set_outline_color(Color::rgb(0, 0, 0)); // Black outline
        delivery_bg.set_position(Vector2f::new(
            self.position.x + 15.0 - delivery_radius,
            self.position.y - 30.0 - delivery_radius
        ));
        window.draw(&delivery_bg);
        
        // We can't directly render text here due to font lifetime issues
        // A proper solution would be to modify the game to maintain fonts
        // and pass them to Colony::render, but for now we'll use shapes
        
        // Draw simple indicators instead
        if self.food_deliveries > 0 {
            let size = delivery_radius * 0.7;
            let mut marker = sfml::graphics::RectangleShape::new();
            marker.set_size(Vector2f::new(size, size));
            marker.set_fill_color(Color::rgb(0, 0, 0)); // Black
            marker.set_position(Vector2f::new(
                self.position.x + 15.0 - size/2.0,
                self.position.y - 30.0 - size/2.0
            ));
            window.draw(&marker);
        }
    }
    
    pub fn get_statistics(&self) -> (usize, f32, usize, u32) {
        (self.ants.len(), self.food_stored, self.max_ants, self.food_deliveries)
    }
    
    pub fn get_position(&self) -> Vector2f {
        self.position
    }
    
    pub fn get_ants(&self) -> &Vec<Ant> {
        &self.ants
    }
    
    pub fn get_food_stored(&self) -> f32 {
        self.food_stored
    }
    
    pub fn set_food_stored(&mut self, food_stored: f32) {
        self.food_stored = food_stored;
    }
    
    pub fn get_max_ants(&self) -> usize {
        self.max_ants
    }
    
    pub fn set_max_ants(&mut self, max_ants: usize) {
        self.max_ants = max_ants;
    }
    
    pub fn get_food_deliveries(&self) -> u32 {
        self.food_deliveries
    }
    
    pub fn set_food_deliveries(&mut self, deliveries: u32) {
        self.food_deliveries = deliveries;
    }
    
    pub fn clear_ants(&mut self) {
        self.ants.clear();
    }
    
    pub fn add_ant(&mut self, ant: Ant) {
        self.ants.push(ant);
    }
} 