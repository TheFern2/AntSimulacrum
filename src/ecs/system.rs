use crate::ecs::world::World;
use crate::ecs::entity::EntityId;
use crate::ecs::component::AntState;
use sfml::graphics::{RenderWindow, CircleShape, RectangleShape, Color, Transformable, Shape, RenderTarget};
use sfml::system::Vector2f;
use std::f32::consts::PI;

/// Trait for all systems
pub trait System {
    /// Update the system
    fn update(&mut self, world: &mut World, delta_time: f32);
    
    /// Get the system name
    fn name(&self) -> &str;
}

/// Basic movement system that updates positions based on velocities
pub struct MovementSystem;

impl MovementSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for MovementSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Get all entities with both position and velocity components
        let entity_ids = world.get_entities_with_components(&[
            super::component::ComponentType::Position,
            super::component::ComponentType::Velocity,
        ]);
        
        for entity_id in entity_ids {
            // First, collect all the data we need
            let position_opt = world.get_component::<super::component::PositionComponent>(
                entity_id,
                super::component::ComponentType::Position,
            );
            
            let velocity_opt = world.get_component::<super::component::VelocityComponent>(
                entity_id,
                super::component::ComponentType::Velocity,
            );
            
            // Calculate the new position
            if let (Some(position), Some(velocity)) = (position_opt, velocity_opt) {
                let new_x = position.x + velocity.dx * velocity.speed * delta_time;
                let new_y = position.y + velocity.dy * velocity.speed * delta_time;
                
                // Now update the position
                if let Some(position) = world.get_component_mut::<super::component::PositionComponent>(
                    entity_id,
                    super::component::ComponentType::Position,
                ) {
                    position.x = new_x;
                    position.y = new_y;
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        "MovementSystem"
    }
}

/// Rendering system that draws all entities with appearance and position components
pub struct RenderingSystem<'a> {
    window: &'a mut RenderWindow,
}

impl<'a> RenderingSystem<'a> {
    pub fn new(window: &'a mut RenderWindow) -> Self {
        Self { window }
    }
}

impl<'a> System for RenderingSystem<'a> {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // Get all entities with both position and appearance components
        let entity_ids = world.get_entities_with_components(&[
            super::component::ComponentType::Position,
            super::component::ComponentType::Appearance,
        ]);
        
        for entity_id in entity_ids {
            // Get position component
            if let Some(position) = world.get_component::<super::component::PositionComponent>(
                entity_id,
                super::component::ComponentType::Position,
            ) {
                // Get appearance component
                if let Some(appearance) = world.get_component::<super::component::AppearanceComponent>(
                    entity_id,
                    super::component::ComponentType::Appearance,
                ) {
                    // Draw the entity based on its appearance
                    match appearance.shape_type {
                        super::component::ShapeType::Circle => {
                            let mut shape = CircleShape::new(appearance.radius, 32);
                            shape.set_position(Vector2f::new(
                                position.x - appearance.radius,
                                position.y - appearance.radius,
                            ));
                            shape.set_fill_color(Color::rgb(
                                appearance.color.0,
                                appearance.color.1,
                                appearance.color.2,
                            ));
                            self.window.draw(&shape);
                        }
                        super::component::ShapeType::Rectangle => {
                            let mut shape = RectangleShape::new();
                            shape.set_size(Vector2f::new(appearance.width, appearance.height));
                            shape.set_position(Vector2f::new(
                                position.x - appearance.width / 2.0,
                                position.y - appearance.height / 2.0,
                            ));
                            shape.set_fill_color(Color::rgb(
                                appearance.color.0,
                                appearance.color.1,
                                appearance.color.2,
                            ));
                            self.window.draw(&shape);
                        }
                    }
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        "RenderingSystem"
    }
}

/// System that handles ant behavior
pub struct AntBehaviorSystem {
    // We'll no longer store window dimensions directly, as we'll get them from environment
}

impl AntBehaviorSystem {
    pub fn new() -> Self {
        Self {}
    }
    
    // Calculate direction to target position
    fn direction_to(&self, from_x: f32, from_y: f32, to_x: f32, to_y: f32) -> f32 {
        let dx = to_x - from_x;
        let dy = to_y - from_y;
        dy.atan2(dx)
    }
}

impl System for AntBehaviorSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Get all entities with ant state, position, and velocity components
        let entity_ids = world.get_entities_with_components(&[
            super::component::ComponentType::AntState,
            super::component::ComponentType::Position,
            super::component::ComponentType::Velocity,
        ]);
        
        // Get environment dimensions for boundary checking
        let (env_width, env_height) = if let Some(environment) = world.get_resource::<crate::environment::Environment>() {
            (environment.get_width() as f32, environment.get_height() as f32)
        } else {
            // Fallback to some default dimensions if environment isn't available
            (800.0, 600.0)
        };

        for entity_id in entity_ids {
            // First, read the current state
            let position_opt = world.get_component::<super::component::PositionComponent>(
                entity_id,
                super::component::ComponentType::Position,
            );
            
            let ant_state_opt = world.get_component::<super::component::AntStateComponent>(
                entity_id,
                super::component::ComponentType::AntState,
            );
            
            let velocity_opt = world.get_component::<super::component::VelocityComponent>(
                entity_id,
                super::component::ComponentType::Velocity,
            );
            
            // Skip if any component is missing
            if position_opt.is_none() || ant_state_opt.is_none() || velocity_opt.is_none() {
                continue;
            }
            
            // Unwrap the options
            let position = position_opt.unwrap();
            let ant_state = ant_state_opt.unwrap();
            let velocity = velocity_opt.unwrap();
            
            // Calculate updates to be applied
            
            // Update timers
            let mut new_pheromone_timer = ant_state.pheromone_timer - delta_time;
            let mut new_random_direction_timer = ant_state.random_direction_timer - delta_time;
            
            // New velocity values
            let mut new_direction = velocity.direction;
            let mut new_dx = velocity.dx;
            let mut new_dy = velocity.dy;
            
            // New ant state
            let mut new_state = ant_state.state.clone();
            let mut new_carrying_food = ant_state.carrying_food;
            
            // Reset random direction timer if needed
            if new_random_direction_timer <= 0.0 {
                // Random direction change
                new_direction += (rand::random::<f32>() - 0.5) * PI / 2.0;
                new_dx = new_direction.cos();
                new_dy = new_direction.sin();
                
                // Reset timer
                new_random_direction_timer = 0.5 + rand::random::<f32>() * 1.5; // 0.5 to 2.0 seconds
            }
            
            // Handle ant state behavior
            match ant_state.state {
                AntState::Idle => {
                    // Transition to searching for food after a short delay
                    new_state = AntState::SearchingForFood;
                }
                
                AntState::SearchingForFood => {
                    // Here we would check for food nearby and transition to returning home if found
                    // For now, we just wander
                    
                    // Drop pheromones periodically
                    if new_pheromone_timer <= 0.0 {
                        // In a real implementation, we would drop home pheromones here
                        new_pheromone_timer = 0.5; // Reset timer
                    }
                }
                
                AntState::ReturningHome => {
                    // Calculate direction to home
                    let home_direction = self.direction_to(
                        position.x, 
                        position.y, 
                        ant_state.home_position.0, 
                        ant_state.home_position.1
                    );
                    
                    // Gradually adjust ant direction to head home
                    let angle_diff = (home_direction - velocity.direction + PI) % (2.0 * PI) - PI;
                    new_direction += angle_diff * 0.1; // Gradually turn
                    
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                    
                    // Drop food pheromones
                    if new_pheromone_timer <= 0.0 {
                        // In a real implementation, we would drop food pheromones here
                        new_pheromone_timer = 0.3; // Reset timer
                    }
                    
                    // Check if we've reached home
                    let dx = position.x - ant_state.home_position.0;
                    let dy = position.y - ant_state.home_position.1;
                    let dist_sq = dx * dx + dy * dy;
                    
                    if dist_sq < 25.0 { // Within 5 units of home
                        new_carrying_food = false;
                        new_state = AntState::SearchingForFood;
                        
                        // In a real implementation, we would add food to the colony here
                    }
                }
                
                AntState::FollowingPheromone => {
                    // Follow pheromone gradient
                    // This is a simplified placeholder - in a real implementation,
                    // we would check pheromone levels in different directions
                    
                    // Randomly decide to stop following and start searching
                    if rand::random::<f32>() < 0.02 {
                        new_state = AntState::SearchingForFood;
                    }
                }
            }
            
            // Boundary checking - bounce off edges
            let margin = 10.0;
            if position.x < margin {
                new_direction = PI - new_direction;
                new_dx = new_direction.cos();
                new_dy = new_direction.sin();
            } else if position.x > env_width - margin {
                new_direction = PI - new_direction;
                new_dx = new_direction.cos();
                new_dy = new_direction.sin();
            }
            
            if position.y < margin {
                new_direction = -new_direction;
                new_dx = new_direction.cos();
                new_dy = new_direction.sin();
            } else if position.y > env_height - margin {
                new_direction = -new_direction;
                new_dx = new_direction.cos();
                new_dy = new_direction.sin();
            }
            
            // Check for wall collisions - this requires access to the environment
            // In a full implementation, we would either:
            // 1. Have an environment component that can be queried
            // 2. Pass the environment as a parameter to the system
            if let Some(environment) = world.get_resource::<crate::environment::Environment>() {
                // Calculate next position
                let next_x = position.x + new_dx * delta_time * 50.0; // Assuming speed is 50.0
                let next_y = position.y + new_dy * delta_time * 50.0;
                
                // Check if next position would hit a wall
                let (grid_x, grid_y) = environment.screen_to_grid(next_x, next_y);
                if environment.get_cell(grid_x, grid_y) == crate::environment::CellType::Wall {
                    // Determine wall orientation by checking adjacent cells
                    let (current_grid_x, current_grid_y) = environment.screen_to_grid(position.x, position.y);
                    
                    let horizontal_wall = current_grid_x != grid_x && 
                                         environment.get_cell(grid_x, current_grid_y) == crate::environment::CellType::Wall;
                    
                    let vertical_wall = current_grid_y != grid_y && 
                                       environment.get_cell(current_grid_x, grid_y) == crate::environment::CellType::Wall;
                    
                    if horizontal_wall {
                        // Bounce horizontally (like a side wall)
                        new_direction = PI - new_direction;
                    } else if vertical_wall {
                        // Bounce vertically (like a floor/ceiling)
                        new_direction = -new_direction;
                    } else {
                        // Corner collision or diagonal approach, reverse direction
                        new_direction = (new_direction + PI) % (2.0 * PI);
                    }
                    
                    // Add small random variation to prevent getting stuck
                    new_direction += (rand::random::<f32>() - 0.5) * 0.2;
                    
                    // Update movement vector
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                }
                
                // Update boundary checking with actual environment dimensions
                let env_width = environment.get_width() as f32;
                let env_height = environment.get_height() as f32;
                
                if position.x < margin {
                    new_direction = PI - new_direction;
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                } else if position.x > env_width - margin {
                    new_direction = PI - new_direction;
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                }
                
                if position.y < margin {
                    new_direction = -new_direction;
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                } else if position.y > env_height - margin {
                    new_direction = -new_direction;
                    new_dx = new_direction.cos();
                    new_dy = new_direction.sin();
                }
            }
            
            // Now apply all the state changes
            if let Some(velocity) = world.get_component_mut::<super::component::VelocityComponent>(
                entity_id,
                super::component::ComponentType::Velocity,
            ) {
                velocity.direction = new_direction;
                velocity.dx = new_dx;
                velocity.dy = new_dy;
            }
            
            if let Some(ant_state) = world.get_component_mut::<super::component::AntStateComponent>(
                entity_id,
                super::component::ComponentType::AntState,
            ) {
                ant_state.state = new_state;
                ant_state.carrying_food = new_carrying_food;
                ant_state.pheromone_timer = new_pheromone_timer;
                ant_state.random_direction_timer = new_random_direction_timer;
            }
        }
    }
    
    fn name(&self) -> &str {
        "AntBehaviorSystem"
    }
}

// Example system, to be expanded later
pub struct ExampleSystem;

impl System for ExampleSystem {
    fn update(&mut self, _world: &mut World, _delta_time: f32) {
        // Example system implementation goes here
    }
    
    fn name(&self) -> &str {
        "ExampleSystem"
    }
} 