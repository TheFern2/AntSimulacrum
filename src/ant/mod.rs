use sfml::graphics::{RenderWindow, RenderTarget, CircleShape, Color, Transformable, Shape};
use sfml::system::Vector2f;
use log::{debug, info, warn};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::environment::{Environment, CellType};
use crate::pheromone::PheromoneType;

// Helper function to convert radians to degrees for easier reading in logs
fn rad_to_deg(rad: f32) -> f32 {
    (rad * 180.0 / std::f32::consts::PI).round()
}

// Position and timestamp history for detecting circular patterns
struct PositionRecord {
    position: Vector2f,
    time: f32,
}

// Global atomic counter for ant IDs
static NEXT_ANT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Ant {
    position: Vector2f,
    direction: f32,  // in radians
    speed: f32,
    carrying_food: bool,
    home_position: Vector2f,
    pheromone_deposit_timer: f32,
    ignore_pheromones_timer: f32, // Timer to ignore pheromones after finding food
    id: usize, // Add ID for tracking individual ants in logs
    position_history: VecDeque<PositionRecord>, // Track recent positions to detect circles
    lifetime: f32, // Track total lifetime of ant
    last_position_record: f32, // Time since last position recording
}

impl Ant {
    // How often to record position for circle detection
    const POSITION_RECORD_INTERVAL: f32 = 0.5;
    // How many position records to keep
    const POSITION_HISTORY_SIZE: usize = 20;
    // Distance threshold for considering a potential circle (grid cells)
    const CIRCLE_DETECTION_THRESHOLD: f32 = 15.0;  // Further reduced from 20.0 to 15.0 to detect circles even sooner
    
    pub fn new(x: f32, y: f32) -> Self {
        // Get a unique ID using the atomic counter
        let id = NEXT_ANT_ID.fetch_add(1, Ordering::SeqCst);
        
        let new_ant = Self {
            position: Vector2f::new(x, y),
            direction: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
            speed: 20.0,  // pixels per second
            carrying_food: false,
            home_position: Vector2f::new(x, y),
            pheromone_deposit_timer: 0.0,
            ignore_pheromones_timer: 0.0,
            id,
            position_history: VecDeque::with_capacity(Self::POSITION_HISTORY_SIZE),
            lifetime: 0.0,
            last_position_record: 0.0,
        };
        
        debug!("Created new ant #{} at position ({:.1},{:.1})", id, x, y);
        new_ant
    }
    
    pub fn update(&mut self, delta_time: f32, environment: &mut Environment) {
        // Update timers
        self.pheromone_deposit_timer -= delta_time;
        self.ignore_pheromones_timer -= delta_time;
        self.lifetime += delta_time;
        self.last_position_record += delta_time;
        
        // Restore normal speed if we had reduced it to escape a circle
        if self.ignore_pheromones_timer <= 2.0 && self.speed < 20.0 {
            self.speed = 20.0;
            debug!("Ant #{} restored normal speed", self.id);
        }
        
        // Record position at regular intervals for ants carrying food
        if self.carrying_food && self.last_position_record >= Self::POSITION_RECORD_INTERVAL {
            self.position_history.push_back(PositionRecord {
                position: self.position,
                time: self.lifetime,
            });
            
            // Keep the history size limited
            if self.position_history.len() > Self::POSITION_HISTORY_SIZE {
                self.position_history.pop_front();
            }
            
            self.last_position_record = 0.0;
            
            // Check for circular patterns
            self.detect_circles();
        }
        
        // Log position for ants carrying food
        if self.carrying_food {
            // Calculate distance to home
            let dx = self.position.x - self.home_position.x;
            let dy = self.position.y - self.home_position.y;
            let distance_to_home = (dx*dx + dy*dy).sqrt();
            
            // Calculate angle to home
            let angle_to_home = dy.atan2(dx);
            let home_direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
            
            // Calculate difference between current direction and home direction
            let angle_diff = (home_direction - self.direction + std::f32::consts::PI * 3.0) % 
                            (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
            
            debug!(
                "Ant #{} [CARRYING FOOD] pos=({:.1},{:.1}) dir={:.0}° home=({:.1},{:.1}) dist_home={:.1} angle_diff={:.0}° ignore_phero={:.1}s",
                self.id,
                self.position.x, self.position.y,
                rad_to_deg(self.direction),
                self.home_position.x, self.home_position.y,
                distance_to_home,
                rad_to_deg(angle_diff),
                self.ignore_pheromones_timer
            );
        }
        
        // Deposit pheromones every so often
        if self.pheromone_deposit_timer <= 0.0 {
            // Reset timer - shorter frequency to create better trails
            self.pheromone_deposit_timer = 0.5; // deposit more frequently
            
            // Deposit appropriate pheromone based on state
            // Food pheromones when carrying food, Home pheromones when searching
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
            
            if self.carrying_food {
                debug!(
                    "Ant #{} deposited FOOD pheromone at ({:.1},{:.1}) with strength {:.1}",
                    self.id, self.position.x, self.position.y, strength
                );
            }
        }
        
        // When carrying food, head directly home more often and make it stronger
        if self.carrying_food {
            // Calculate direction to home
            let dx = self.position.x - self.home_position.x;
            let dy = self.position.y - self.home_position.y;
            let dist_sq = dx*dx + dy*dy;
            
            if dist_sq > 0.1 { // Any distance from home
                let angle_to_home = dy.atan2(dx);
                // Add PI to point toward home
                let home_direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
                
                // Direct homing chance increases when ignoring pheromones
                let homing_factor = if self.ignore_pheromones_timer > 0.0 {
                    // Go straight home for a while after finding food
                    0.75  // 75% home direction + 25% current direction 
                } else if rand::random::<f32>() < 0.35 {  // Increased from 0.25 to 0.35
                    // Random chance for strong homing
                    0.6   // 60% home direction + 40% current direction
                } else {
                    // Normal following with slight home bias
                    0.2   // 20% home direction + 80% current direction
                };
                
                // Blend current direction with home direction
                let angle_diff = (home_direction - self.direction + std::f32::consts::PI * 3.0) % 
                                (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
                self.direction += angle_diff * homing_factor;
                
                // Add small random variation to prevent perfect straight lines that might lead to circles
                self.direction += (rand::random::<f32>() - 0.5) * 0.1;
            }
        }
        
        // Reduced random movement chance - let pheromone following be more dominant
        if rand::random::<f32>() < 0.05 * delta_time {
            self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
        } 
        // Only follow pheromones if not in ignore state
        else if self.ignore_pheromones_timer <= 0.0 && rand::random::<f32>() < 0.9 {
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
        if environment.get_cell(grid_x, grid_y) == CellType::Wall {
            // Hit a wall, bounce off in a realistic way
            
            // Check which direction we need to bounce (horizontal or vertical wall)
            // Try checking horizontal and vertical adjacent cells to determine wall orientation
            let (current_grid_x, current_grid_y) = environment.screen_to_grid(self.position.x, self.position.y);
            
            let horizontal_wall = current_grid_x != grid_x && 
                                 environment.get_cell(grid_x, current_grid_y) == CellType::Wall;
            
            let vertical_wall = current_grid_y != grid_y && 
                               environment.get_cell(current_grid_x, grid_y) == CellType::Wall;
            
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
            8  // Further reduced from 10 to 8 to make behavior much less twitchy
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
                std::f32::consts::PI * 0.6  // Further reduced from 0.7 to 0.6 to be even more forward-focused
            } else {
                // More restricted angle when searching for food
                std::f32::consts::PI * 2.0/3.0
            };
            
            if angle_diff.abs() < forward_angle_limit {
                // Gradually turn towards the best direction
                // Reduced turn rate for carrying ants to make movement more stable
                let turn_rate = if self.carrying_food {
                    0.15  // Further reduced from 0.2 to 0.15 to make turns extremely gradual
                } else {
                    0.7  // Standard turn rate for foraging
                };
                
                let old_direction = self.direction;
                self.direction += angle_diff * turn_rate;
                
                if self.carrying_food {
                    debug!(
                        "Ant #{} following pheromone - old_dir={:.0}° new_dir={:.0}° diff={:.0}° turn_rate={:.1}",
                        self.id, 
                        rad_to_deg(old_direction), 
                        rad_to_deg(self.direction),
                        rad_to_deg(angle_diff),
                        turn_rate
                    );
                }
            } else if rand::random::<f32>() < 0.1 {
                // Small chance to make a big turn anyway, to avoid getting stuck
                self.direction += angle_diff * 0.4;
                
                if self.carrying_food {
                    debug!(
                        "Ant #{} made BIG TURN to avoid getting stuck, new_dir={:.0}°",
                        self.id, 
                        rad_to_deg(self.direction)
                    );
                }
            }
            
            // Add a small random variation to prevent perfect following that might lead to circles
            // Use smaller variation for carrying ants to prevent erratic movement
            let random_variation = if self.carrying_food {
                (rand::random::<f32>() - 0.5) * 0.03  // Further reduced from 0.05 to 0.03
            } else {
                (rand::random::<f32>() - 0.5) * 0.2  // Standard random variation
            };
            self.direction += random_variation;
        } else {
            // If no pheromone found, increase random movement slightly
            // Higher chance of direction change when carrying food to escape local minima
            let random_chance = if self.carrying_food { 0.7 } else { 0.4 };  // Increased from 0.6 to 0.7
            if rand::random::<f32>() < random_chance {
                let old_direction = self.direction;
                let dir_change = if self.carrying_food {
                    (rand::random::<f32>() - 0.5) * std::f32::consts::PI * 0.6  // Increased from 0.5 to 0.6
                } else {
                    (rand::random::<f32>() - 0.5) * std::f32::consts::PI * 0.5
                };
                self.direction += dir_change;
                
                if self.carrying_food {
                    debug!(
                        "Ant #{} NO PHEROMONE FOUND - random turn from {:.0}° to {:.0}° (change: {:.0}°)",
                        self.id, 
                        rad_to_deg(old_direction), 
                        rad_to_deg(self.direction),
                        rad_to_deg(dir_change)
                    );
                }
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
            0.005 // Further reduced from 0.01 to 0.005 to detect even weaker home trails
        } else {
            0.05 // Standard threshold for other situations
        };
        
        let mut best_strength_found = best_strength;
        let mut best_direction = None;
        
        // Define arrays outside the loop to avoid temporary value errors
        let carrying_points = [
            min_sense_distance, 
            sense_distance * 0.4,  // Reduced from 0.5 to 0.4
            sense_distance * 0.7,  // Added middle distance point
            sense_distance
        ];
        
        let standard_points = [
            min_sense_distance, 
            sense_distance * 0.25, 
            sense_distance * 0.5, 
            sense_distance * 0.75, 
            sense_distance
        ];
        
        // For logging purposes
        let mut detected_pheromones = Vec::new();
        
        // Check in multiple directions
        for i in 0..num_directions {
            // Calculate angle relative to current direction, spread evenly around the circle
            let angle = (i as f32 / num_directions as f32) * 2.0 * std::f32::consts::PI;
            
            // For carrying ants, bias sampling toward forward angles
            // This makes them less likely to backtrack
            let biased_angle = if self.carrying_food {
                // Concentrate sampling in the forward 180 degrees (avoiding going backward)
                // Map i from [0..num_directions] to [-PI/2..PI/2]
                (i as f32 / num_directions as f32 - 0.5) * std::f32::consts::PI
            } else {
                angle
            };
            
            // Add to current direction to get world angle
            let world_angle = (self.direction + biased_angle) % (2.0 * std::f32::consts::PI);
            
            // If carrying food, don't check directions that would make the ant turn back
            if self.carrying_food {
                let angle_diff = (world_angle - self.direction).abs() % (2.0 * std::f32::consts::PI);
                let back_angle = std::f32::consts::PI * 0.5; // Further reduced from 0.6 to 0.5 to focus even more forward
                
                if angle_diff > back_angle && angle_diff < (2.0 * std::f32::consts::PI - back_angle) {
                    continue; // Skip this direction - it would make the ant turn back too much
                }
            }
            
            // Sample more points when carrying food
            let sample_points = if self.carrying_food {
                &carrying_points[..]
            } else {
                &standard_points[..]
            };
            
            // Check at different distances, using more sample points
            for d in sample_points {
                let check_x = self.position.x + world_angle.cos() * d;
                let check_y = self.position.y + world_angle.sin() * d;
                
                let strength = environment.pheromone_system_ref().get_pheromone(check_x, check_y, &pheromone_type);
                
                if strength > best_strength {
                    // For logging - save all detected pheromones above threshold
                    if self.carrying_food {
                        detected_pheromones.push((world_angle, strength, *d, check_x, check_y));
                    }
                }
                
                // Add a bias to favor forward directions for carrying ants
                let direction_bias = if self.carrying_food {
                    // Calculate how "forward" this direction is (1.0 = directly forward, 0.0 = directly backward)
                    let forward_factor = ((world_angle - self.direction + std::f32::consts::PI).abs() 
                                        % (2.0 * std::f32::consts::PI) - std::f32::consts::PI).abs() / std::f32::consts::PI;
                    forward_factor * 0.03 // Further reduced from 0.05 to 0.03
                } else {
                    0.0
                };
                
                // Apply the directional bias to the perceived strength
                let biased_strength = strength + direction_bias;
                
                if biased_strength > best_strength_found {
                    best_strength_found = biased_strength;
                    best_direction = Some(world_angle);
                }
            }
        }
        
        // Log pheromone detection details for ants carrying food
        if self.carrying_food && !detected_pheromones.is_empty() {
            // Sort pheromones by strength (descending)
            detected_pheromones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Log the strongest pheromones (up to 3)
            let log_limit = std::cmp::min(3, detected_pheromones.len());
            for i in 0..log_limit {
                let (angle, strength, distance, x, y) = detected_pheromones[i];
                debug!(
                    "Ant #{} detected {} pheromone at ({:.1},{:.1}) strength={:.3} angle={:.0}° dist={:.1} ",
                    self.id,
                    match pheromone_type {
                        PheromoneType::Home => "HOME",
                        PheromoneType::Food => "FOOD",
                    },
                    x, y, strength, rad_to_deg(angle), distance
                );
            }
            
            // Log the chosen direction
            if let Some(direction) = best_direction {
                debug!(
                    "Ant #{} chose direction {:.0}° (current={:.0}°, diff={:.0}°)",
                    self.id, 
                    rad_to_deg(direction), 
                    rad_to_deg(self.direction),
                    rad_to_deg((direction - self.direction + std::f32::consts::PI * 3.0) % 
                              (std::f32::consts::PI * 2.0) - std::f32::consts::PI)
                );
            } else {
                debug!("Ant #{} found no suitable pheromones to follow", self.id);
            }
        }
        
        best_direction
    }
    
    fn check_for_food(&mut self, environment: &mut Environment) {
        // Get grid coordinates
        let (grid_x, grid_y) = environment.screen_to_grid(self.position.x, self.position.y);
        
        // Check if we're at a food source and not carrying food
        if !self.carrying_food && environment.get_cell(grid_x, grid_y) == CellType::Food {
            // Take some food
            self.carrying_food = true;
            
            // Set a timer to temporarily ignore pheromones after finding food
            // This will help prevent ants from getting stuck in circles
            self.ignore_pheromones_timer = 10.0; // Increased from 8.0 to 10.0 seconds
            
            // Don't deposit any pheromones at the food location
            // This prevents creating any kind of attraction point
            
            // Force point directly toward home instead of reversing
            let dx = self.position.x - self.home_position.x;
            let dy = self.position.y - self.home_position.y;
            let distance_to_home = (dx*dx + dy*dy).sqrt();
            
            // Calculate angle to home and set direction directly
            let angle_to_home = dy.atan2(dx);
            // Add PI to point toward home
            self.direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
            
            // Add a small random variation to prevent ants from all taking the same path
            let angle_variation = (rand::random::<f32>() - 0.5) * 0.3;
            self.direction += angle_variation;
            
            // Move significantly away from the food immediately to escape the food's "gravity well"
            let escape_distance = 30.0; // Doubled from 15.0
            self.position.x += self.direction.cos() * escape_distance;
            self.position.y += self.direction.sin() * escape_distance;
            
            info!(
                "Ant #{} FOUND FOOD at ({},{}) pos=({:.1},{:.1}) distance_home={:.1} heading={:.0}° variation={:.0}° escape={:.1}",
                self.id, grid_x, grid_y, 
                self.position.x, self.position.y,
                distance_to_home,
                rad_to_deg(self.direction),
                rad_to_deg(angle_variation),
                escape_distance
            );
        }
        
        // Check if we're at the nest and carrying food
        else if self.carrying_food && environment.get_cell(grid_x, grid_y) == CellType::AntNest {
            // Deposit food
            self.carrying_food = false;
            
            println!("DEBUG: Ant #{} attempting to deposit food at nest at ({},{}) pos=({:.1},{:.1})", 
                self.id, grid_x, grid_y, self.position.x, self.position.y);
            
            // Get the center of the current nest cell
            let _nest_center_x = (grid_x as f32 + 0.5) * 10.0; // Assuming cell size is 10.0
            let _nest_center_y = (grid_y as f32 + 0.5) * 10.0;
            
            // Signal the colony to add food
            let mut delivered = false;
            println!("DEBUG: Number of colonies: {}", environment.get_colonies().len());
            
            for colony in environment.get_colonies().iter_mut() {
                // Check if we're at this colony's position
                let colony_pos = colony.get_position();
                let dx = self.position.x - colony_pos.x;
                let dy = self.position.y - colony_pos.y;
                let distance_squared = dx * dx + dy * dy;
                
                println!("DEBUG: Ant #{} checking colony at ({:.1},{:.1}), distance_squared={:.1}, checking radius={:.1}", 
                    self.id, colony_pos.x, colony_pos.y, distance_squared, 35.0 * 35.0);
                
                if distance_squared < 35.0 * 35.0 {  // If within 35 pixels of colony
                    colony.add_food(1.0);  // Add 1 unit of food
                    delivered = true;
                    info!(
                        "Ant #{} DELIVERED FOOD to nest at ({},{}) pos=({:.1},{:.1})",
                        self.id, grid_x, grid_y, self.position.x, self.position.y
                    );
                    break;
                }
            }
            
            if !delivered {
                println!("DEBUG: WARNING - Ant #{} at nest but couldn't find nearby colony to deliver food!", self.id);
                
                // Try again with relaxed distance check
                let mut closest_colony = None;
                let mut closest_distance = f32::MAX;
                
                for (i, colony) in environment.get_colonies().iter_mut().enumerate() {
                    let colony_pos = colony.get_position();
                    let dx = self.position.x - colony_pos.x;
                    let dy = self.position.y - colony_pos.y;
                    let distance_squared = dx * dx + dy * dy;
                    
                    if distance_squared < closest_distance {
                        closest_distance = distance_squared;
                        closest_colony = Some((i, colony));
                    }
                }
                
                // Use the closest colony if one exists
                if let Some((i, colony)) = closest_colony {
                    colony.add_food(1.0);
                    println!("DEBUG: Ant #{} delivered to closest colony #{} at distance {:.1}", 
                        self.id, i, closest_distance.sqrt());
                    info!(
                        "Ant #{} DELIVERED FOOD to closest colony #{} at ({},{}) pos=({:.1},{:.1})",
                        self.id, i, grid_x, grid_y, self.position.x, self.position.y
                    );
                    delivered = true;
                } else if let Some(colony) = environment.get_colonies().first_mut() {
                    // Fall back to the first colony as last resort
                    colony.add_food(1.0);
                    info!(
                        "Ant #{} DELIVERED FOOD to default colony! nest=({},{}) pos=({:.1},{:.1}) colony_pos=({:.1},{:.1})",
                        self.id, grid_x, grid_y, self.position.x, self.position.y, 
                        colony.get_position().x, colony.get_position().y
                    );
                    delivered = true;
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
            
            // Add a small random variation when leaving nest
            self.direction += (rand::random::<f32>() - 0.5) * 0.5;
        }
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        // Create main body
        let mut ant_body = CircleShape::new(5.0, 8);
        ant_body.set_position(Vector2f::new(self.position.x - 5.0, self.position.y - 5.0));
        
        if self.carrying_food {
            ant_body.set_fill_color(Color::rgb(255, 210, 0)); // Bright gold/yellow
        } else {
            ant_body.set_fill_color(Color::rgb(70, 70, 70)); // Darker grey
        }
        
        // Create head to show direction
        let mut ant_head = CircleShape::new(3.0, 6);
        let head_x = self.position.x + self.direction.cos() * 7.0;
        let head_y = self.position.y + self.direction.sin() * 7.0;
        ant_head.set_position(Vector2f::new(head_x - 3.0, head_y - 3.0));
        
        // Also change head color based on state
        if self.carrying_food {
            ant_head.set_fill_color(Color::rgb(200, 100, 0)); // Orange head when carrying food
        } else {
            ant_head.set_fill_color(Color::rgb(20, 20, 20)); // Black head normally
        }
        
        // Draw a small colored dot if ignoring pheromones
        if self.ignore_pheromones_timer > 0.0 {
            let mut indicator = CircleShape::new(2.0, 4);
            indicator.set_position(Vector2f::new(self.position.x - 2.0, self.position.y - 2.0));
            indicator.set_fill_color(Color::rgb(255, 0, 0)); // Red dot
            window.draw(&indicator);
        }
        
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
    
    pub fn get_ignore_pheromones_timer(&self) -> f32 {
        self.ignore_pheromones_timer
    }
    
    pub fn set_ignore_pheromones_timer(&mut self, timer: f32) {
        self.ignore_pheromones_timer = timer;
    }

    // Helper method to detect if an ant is moving in circles
    fn detect_circles(&mut self) {
        // Need at least a few points to detect a circle
        if self.position_history.len() < 5 {
            return;
        }
        
        // Check if we're close to any previous position from more than 2 seconds ago
        let current_pos = self.position;
        let current_time = self.lifetime;
        
        for record in self.position_history.iter() {
            // Only compare positions that are at least 2 seconds old
            if current_time - record.time > 2.0 {
                let dx = current_pos.x - record.position.x;
                let dy = current_pos.y - record.position.y;
                let distance = (dx*dx + dy*dy).sqrt();
                
                // If we're close to a past position, we might be circling
                if distance < Self::CIRCLE_DETECTION_THRESHOLD {
                    // Calculate how long we've been circling
                    let circle_time = current_time - record.time;
                    
                    warn!(
                        "CIRCULAR MOVEMENT DETECTED: Ant #{} near past position from {:.1}s ago. Current=({:.1},{:.1}) Past=({:.1},{:.1}) Distance={:.1}",
                        self.id,
                        circle_time,
                        current_pos.x, current_pos.y,
                        record.position.x, record.position.y,
                        distance
                    );
                    
                    // If ant is circling for too long, force it to escape
                    if circle_time > 3.0 {  // Further reduced from 4.0 to 3.0 to break out of circles even sooner
                        // Clear position history to avoid multiple detections
                        self.position_history.clear();

                        if self.carrying_food {
                            // Force the ant to ignore pheromones for a while
                            self.ignore_pheromones_timer = 15.0;  // Increased from 12.0 to 15.0
                            
                            // Temporarily reduce speed to break out of circles
                            self.speed = 8.0; // Further reduced from 10.0 to 8.0
                            
                            // Calculate direction to home
                            let dx = self.position.x - self.home_position.x;
                            let dy = self.position.y - self.home_position.y;
                            let angle_to_home = dy.atan2(dx);
                            
                            // Set direction home with some randomness
                            let old_direction = self.direction;
                            self.direction = (angle_to_home + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
                            
                            // Add larger random variation to escape the circle pattern
                            self.direction += (rand::random::<f32>() - 0.5) * 0.8; // Increased from 0.6 to 0.8
                            
                            // Move a bit in the new direction immediately to escape the circle
                            let escape_step = 15.0; // Increased from 10.0 to 15.0
                            self.position.x += self.direction.cos() * escape_step;
                            self.position.y += self.direction.sin() * escape_step;
                            
                            warn!(
                                "CIRCLE ESCAPE (HOME): Ant #{} - changing direction from {:.0}° to {:.0}° and reducing speed to {:.1}",
                                self.id,
                                rad_to_deg(old_direction),
                                rad_to_deg(self.direction),
                                self.speed
                            );
                            
                            // Schedule speed restoration after some time (using a timer system)
                            // We'll restore speed after 5 seconds of the ignore_pheromones_timer
                            // We don't need a new timer as we can use the existing ignore_pheromones_timer
                        } else {
                            // If not carrying food, perform a large random turn and ignore pheromones
                            self.ignore_pheromones_timer = 7.0; // Increased from 5.0 to 7.0

                            let old_direction = self.direction;
                            // Add a random turn up to +/- 90 degrees (PI radians)
                            self.direction += (rand::random::<f32>() - 0.5) * std::f32::consts::PI; 
                            self.direction = (self.direction + 2.0 * std::f32::consts::PI) % (2.0 * std::f32::consts::PI); // Ensure positive angle

                            warn!(
                                "CIRCLE ESCAPE (SEARCHING): Ant #{} - changing direction from {:.0}° to {:.0}°",
                                self.id,
                                rad_to_deg(old_direction),
                                rad_to_deg(self.direction)
                            );
                        }
                        
                        return; // Exit after finding one circle
                    }
                }
            }
        }
    }
} 