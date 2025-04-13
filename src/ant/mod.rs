use sfml::graphics::{RenderWindow, RenderTarget, CircleShape, Color, Transformable, Shape};
use sfml::system::Vector2f;

use crate::environment::Environment;
use crate::pheromone::PheromoneType;

pub struct Ant {
    position: Vector2f,
    direction: f32,  // in radians
    speed: f32,
    carrying_food: bool,
    home_position: Vector2f,
    pheromone_deposit_timer: f32,
}

impl Ant {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vector2f::new(x, y),
            direction: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
            speed: 20.0,  // pixels per second
            carrying_food: false,
            home_position: Vector2f::new(x, y),
            pheromone_deposit_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32, environment: &mut Environment) {
        // Update pheromone deposit timer
        self.pheromone_deposit_timer -= delta_time;
        
        // Deposit pheromones every so often
        if self.pheromone_deposit_timer <= 0.0 {
            // Reset timer - shorter frequency to create better trails
            self.pheromone_deposit_timer = 0.5; // deposit more frequently
            
            // Deposit appropriate pheromone based on state
            // Food pheromones when carrying food, Home pheromones when searching
            // This was backwards before! (was opposite of what ants actually do)
            let pheromone_type = if self.carrying_food {
                PheromoneType::Food // Leave food trail when carrying food
            } else {
                PheromoneType::Home // Leave home trail when searching
            };
            
            // Only deposit strong pheromones when actually carrying food or returning to nest
            let strength = if self.carrying_food {
                0.8 // Stronger pheromone when carrying food
            } else {
                0.3 // Medium strength when searching
            };
            
            environment.pheromone_system().add_pheromone(
                self.position.x, 
                self.position.y, 
                pheromone_type, 
                strength
            );
        }
        
        // Let carrying ants have a chance to head toward home directly
        if self.carrying_food && rand::random::<f32>() < 0.1 {
            // Calculate direction to home
            let dx = self.position.x - self.home_position.x;
            let dy = self.position.y - self.home_position.y;
            let dist_sq = dx*dx + dy*dy;
            
            if dist_sq > 100.0 { // Only if we're not super close to home already
                let angle_to_home = dy.atan2(dx);
                // Add PI to point toward home
                let home_direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
                
                // Gradually turn toward home
                let angle_diff = (home_direction - self.direction + std::f32::consts::PI * 3.0) % 
                                (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
                self.direction += angle_diff * 0.2;
            }
        }
        
        // Reduced random movement chance - let pheromone following be more dominant
        if rand::random::<f32>() < 0.05 * delta_time {
            self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
        } 
        // Increased chance to follow pheromones
        else if rand::random::<f32>() < 0.9 { // 90% chance to follow pheromones (increased from 80%)
            self.follow_pheromones(environment);
        }
        
        // Interact with the environment
        self.check_for_food(environment);
        
        // Calculate next position
        let dx = self.direction.cos() * self.speed * delta_time;
        let dy = self.direction.sin() * self.speed * delta_time;
        let next_x = self.position.x + dx;
        let next_y = self.position.y + dy;
        
        // Check for wall collisions
        let (grid_x, grid_y) = environment.screen_to_grid(next_x, next_y);
        if environment.get_cell(grid_x, grid_y) == crate::environment::CellType::Wall {
            // Hit a wall, bounce off in a realistic way
            
            // Check which direction we need to bounce (horizontal or vertical wall)
            // Try checking horizontal and vertical adjacent cells to determine wall orientation
            let (current_grid_x, current_grid_y) = environment.screen_to_grid(self.position.x, self.position.y);
            
            let horizontal_wall = current_grid_x != grid_x && 
                                 environment.get_cell(grid_x, current_grid_y) == crate::environment::CellType::Wall;
            
            let vertical_wall = current_grid_y != grid_y && 
                               environment.get_cell(current_grid_x, grid_y) == crate::environment::CellType::Wall;
            
            if horizontal_wall {
                // Bounce horizontally
                self.direction = std::f32::consts::PI - self.direction;
            } else if vertical_wall {
                // Bounce vertically
                self.direction = -self.direction;
            } else {
                // Corner or diagonal collision, reverse direction
                self.direction = (self.direction + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
            }
            
            // Add a small random variation to prevent ants from getting stuck
            self.direction += (rand::random::<f32>() - 0.5) * 0.2;
            
            // Move in the new direction
            let new_dx = self.direction.cos() * self.speed * delta_time;
            let new_dy = self.direction.sin() * self.speed * delta_time;
            self.position.x += new_dx;
            self.position.y += new_dy;
        } else {
            // No wall, proceed with movement
            self.position.x = next_x;
            self.position.y = next_y;
        }
        
        // Boundary check - bounce off edges
        let margin = 10.0;
        let env_width = environment.get_width() as f32;
        let env_height = environment.get_height() as f32;
        
        if self.position.x < margin {
            self.position.x = margin;
            self.direction = std::f32::consts::PI - self.direction;
        } else if self.position.x > env_width - margin {
            self.position.x = env_width - margin;
            self.direction = std::f32::consts::PI - self.direction;
        }
        
        if self.position.y < margin {
            self.position.y = margin;
            self.direction = -self.direction;
        } else if self.position.y > env_height - margin {
            self.position.y = env_height - margin;
            self.direction = -self.direction;
        }
    }
    
    fn random_direction_change(&mut self) -> bool {
        // Random direction change, 10% chance per update
        if rand::random::<f32>() < 0.1 {
            self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI / 2.0;
            return true;
        }
        false
    }
    
    fn follow_pheromones(&mut self, environment: &Environment) {
        // Determine which pheromone to follow based on current state
        // When carrying food, follow Home pheromones to return home
        // When not carrying food, follow Food pheromones to find food
        let pheromone_type = if self.carrying_food {
            PheromoneType::Home  // Follow home trails when carrying food
        } else {
            PheromoneType::Food  // Follow food trails when searching
        };
        
        // Check pheromones in multiple directions
        let num_directions = if self.carrying_food {
            18  // More sampling directions for carrying ants to better find way home
        } else {
            12  // Standard number of directions for foraging
        };
        
        let best_direction = self.find_strongest_pheromone_direction(environment, pheromone_type, num_directions);
        
        // If we found a direction with pheromones, adjust our direction towards it
        if let Some(best_dir) = best_direction {
            // Calculate the angle difference between current direction and pheromone direction
            let angle_diff = (best_dir - self.direction + std::f32::consts::PI * 3.0) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
            
            // Improved logic to prevent circular trails
            // Only turn if the pheromone is roughly ahead of us (wider angle when carrying food)
            let forward_angle_limit = if self.carrying_food {
                // Allow wider angle consideration when carrying food (nearly all directions)
                std::f32::consts::PI * 0.9  
            } else {
                // More restricted angle when searching for food
                std::f32::consts::PI * 2.0/3.0
            };
            
            if angle_diff.abs() < forward_angle_limit {
                // Gradually turn towards the best direction
                // Increased turn rate for carrying ants to make return more effective
                let turn_rate = if self.carrying_food {
                    0.8  // Turn more aggressively when carrying food
                } else {
                    0.7  // Standard turn rate for foraging
                };
                self.direction += angle_diff * turn_rate;
            } else if rand::random::<f32>() < 0.1 {
                // Small chance to make a big turn anyway, to avoid getting stuck
                self.direction += angle_diff * 0.4;
            }
            
            // Add a small random variation to prevent perfect following that might lead to circles
            self.direction += (rand::random::<f32>() - 0.5) * 0.2;
        } else {
            // If no pheromone found, increase random movement slightly
            // Higher chance of direction change when carrying food to escape local minima
            let random_chance = if self.carrying_food { 0.5 } else { 0.4 };
            if rand::random::<f32>() < random_chance {
                self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI * 0.5;
            }
        }
    }
    
    fn find_strongest_pheromone_direction(
        &self,
        environment: &Environment,
        pheromone_type: PheromoneType,
        num_directions: usize
    ) -> Option<f32> {
        let sense_distance = 40.0; // Increased sensing distance for better trail finding
        let min_sense_distance = 5.0; // Reduced minimum distance to better sense nearby trails
        
        // Lower threshold for Home pheromones when carrying food, to make it easier to find way home
        let best_strength = if self.carrying_food && pheromone_type == PheromoneType::Home {
            0.02 // Very low threshold for detecting home pheromones when carrying food
        } else {
            0.05 // Standard threshold for other situations
        };
        
        let mut best_strength_found = best_strength;
        let mut best_direction = None;
        
        // Use the passed num_directions
        
        // Check in multiple directions
        for i in 0..num_directions {
            let angle = (i as f32 / num_directions as f32) * 2.0 * std::f32::consts::PI;
            
            // Calculate an offset angle to avoid sampling in a perfect grid
            let offset_angle = angle + self.direction.cos() * 0.1;
            
            // Sample more points when carrying food
            let sample_points = if self.carrying_food {
                [min_sense_distance, sense_distance * 0.2, sense_distance * 0.4, 
                 sense_distance * 0.6, sense_distance * 0.8, sense_distance].iter()
            } else {
                [min_sense_distance, sense_distance * 0.25, sense_distance * 0.5, 
                 sense_distance * 0.75, sense_distance].iter()
            };
            
            // Check at different distances, using more sample points
            for d in sample_points {
                let check_x = self.position.x + offset_angle.cos() * d;
                let check_y = self.position.y + offset_angle.sin() * d;
                
                let strength = environment.pheromone_system_ref().get_pheromone(check_x, check_y, &pheromone_type);
                
                if strength > best_strength_found {
                    best_strength_found = strength;
                    best_direction = Some(angle);
                }
            }
        }
        
        best_direction
    }
    
    fn check_for_food(&mut self, environment: &mut Environment) {
        // Get grid coordinates
        let (grid_x, grid_y) = environment.screen_to_grid(self.position.x, self.position.y);
        
        // Check if we're at a food source and not carrying food
        if !self.carrying_food && environment.get_cell(grid_x, grid_y) == crate::environment::CellType::Food {
            // Take some food
            self.carrying_food = true;
            
            // Reverse direction to head back towards nest
            self.direction = (self.direction + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
            
            // Don't deposit food pheromone right at the food - it creates a trap
            // Instead, deposit a HOME pheromone since other ants need to find their way back
            environment.pheromone_system().add_pheromone(
                self.position.x,
                self.position.y,
                PheromoneType::Home,
                0.5 // Medium strength HOME pheromone at food location to help ants get back
            );
            
            // Attempt to point toward home when first finding food
            let dx = self.position.x - self.home_position.x;
            let dy = self.position.y - self.home_position.y;
            if dx*dx + dy*dy > 100.0 { // Only if we're not super close to home already
                let angle_to_home = dy.atan2(dx);
                // Add PI to point toward home
                let home_direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
                // Blend current reversed direction with home direction
                self.direction = self.direction * 0.3 + home_direction * 0.7;
            }
        }
        
        // Check if we're at the nest and carrying food
        else if self.carrying_food && environment.get_cell(grid_x, grid_y) == crate::environment::CellType::AntNest {
            // Deposit food
            self.carrying_food = false;
            
            // Signal the colony to add food
            for colony in environment.get_colonies().iter_mut() {
                // Check if we're at this colony's position
                let colony_pos = colony.get_position();
                let dx = self.position.x - colony_pos.x;
                let dy = self.position.y - colony_pos.y;
                let distance_squared = dx * dx + dy * dy;
                
                if distance_squared < 30.0 * 30.0 {  // If within 30 pixels of colony
                    colony.add_food(1.0);  // Add 1 unit of food
                    break;
                }
            }
            
            // Deposit a stronger pheromone at the nest
            environment.pheromone_system().add_pheromone(
                self.position.x,
                self.position.y,
                PheromoneType::Home,
                0.9 // Strong pheromone at nest location
            );
            
            // Reverse direction to head back out
            self.direction = (self.direction + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
        }
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        // Create main body
        let mut ant_body = CircleShape::new(5.0, 8);
        ant_body.set_position(Vector2f::new(self.position.x - 5.0, self.position.y - 5.0));
        
        if self.carrying_food {
            ant_body.set_fill_color(Color::rgb(200, 200, 0)); // Yellow
        } else {
            ant_body.set_fill_color(Color::rgb(50, 50, 50)); // Dark grey
        }
        
        // Create head to show direction
        let mut ant_head = CircleShape::new(3.0, 6);
        let head_x = self.position.x + self.direction.cos() * 7.0;
        let head_y = self.position.y + self.direction.sin() * 7.0;
        ant_head.set_position(Vector2f::new(head_x - 3.0, head_y - 3.0));
        ant_head.set_fill_color(Color::rgb(20, 20, 20)); // Black
        
        window.draw(&ant_body);
        window.draw(&ant_head);
    }

    // Accessor methods for save/load functionality
    pub fn get_position(&self) -> Vector2f {
        self.position
    }
    
    pub fn get_direction(&self) -> f32 {
        self.direction
    }
    
    pub fn set_direction(&mut self, direction: f32) {
        self.direction = direction;
    }
    
    pub fn get_speed(&self) -> f32 {
        self.speed
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
    
    pub fn is_carrying_food(&self) -> bool {
        self.carrying_food
    }
    
    pub fn set_carrying_food(&mut self, carrying_food: bool) {
        self.carrying_food = carrying_food;
    }
    
    pub fn get_home_position(&self) -> Vector2f {
        self.home_position
    }
    
    pub fn set_home_position(&mut self, home_position: Vector2f) {
        self.home_position = home_position;
    }
    
    pub fn get_pheromone_deposit_timer(&self) -> f32 {
        self.pheromone_deposit_timer
    }
    
    pub fn set_pheromone_deposit_timer(&mut self, timer: f32) {
        self.pheromone_deposit_timer = timer;
    }
} 