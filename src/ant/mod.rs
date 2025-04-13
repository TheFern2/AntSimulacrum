use sfml::graphics::{RenderWindow, RenderTarget, CircleShape, Color, Transformable, Shape};
use sfml::system::Vector2f;

use crate::environment::Environment;

pub struct Ant {
    position: Vector2f,
    direction: f32,  // in radians
    speed: f32,
    carrying_food: bool,
}

impl Ant {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vector2f::new(x, y),
            direction: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
            speed: 20.0,  // pixels per second
            carrying_food: false,
        }
    }
    
    pub fn update(&mut self, delta_time: f32, environment: &mut Environment) {
        // Will implement more sophisticated behavior later
        // For now, just basic wandering
        
        // Random direction change
        if rand::random::<f32>() < 0.1 {
            self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI / 2.0;
        }
        
        // Move forward
        let dx = self.direction.cos() * self.speed * delta_time;
        let dy = self.direction.sin() * self.speed * delta_time;
        self.position.x += dx;
        self.position.y += dy;
        
        // Boundary check - bounce off edges
        let margin = 10.0;
        if self.position.x < margin {
            self.position.x = margin;
            self.direction = std::f32::consts::PI - self.direction;
        } else if self.position.x > 800.0 - margin {
            self.position.x = 800.0 - margin;
            self.direction = std::f32::consts::PI - self.direction;
        }
        
        if self.position.y < margin {
            self.position.y = margin;
            self.direction = -self.direction;
        } else if self.position.y > 600.0 - margin {
            self.position.y = 600.0 - margin;
            self.direction = -self.direction;
        }
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        let mut ant_shape = CircleShape::new(3.0, 3);
        ant_shape.set_position(Vector2f::new(self.position.x - 3.0, self.position.y - 3.0));
        
        if self.carrying_food {
            ant_shape.set_fill_color(Color::rgb(200, 200, 0)); // Yellow
        } else {
            ant_shape.set_fill_color(Color::rgb(0, 0, 0)); // Black
        }
        
        window.draw(&ant_shape);
    }
} 