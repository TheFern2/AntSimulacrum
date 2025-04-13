# Pheromone System Architecture

## Overview

The pheromone system is a core component of Ant Simulacrum that enables realistic ant behavior and emergent path finding. Ants communicate indirectly by depositing and sensing pheromones in the environment, allowing them to collectively find efficient routes to food sources and back to the nest.

## Pheromone Types

The system supports two primary pheromone types:

```rust
pub enum PheromoneType {
    Food,   // Deposited when returning to nest with food
    Home,   // Deposited when searching for food
}
```

## Pheromone Storage and Management

Pheromones are stored in a grid-based system implemented by the `PheromoneSystem` struct:

```rust
pub struct PheromoneSystem {
    // Grid-based pheromone storage
    // Key: (grid_x, grid_y, pheromone_type)
    // Value: strength (0.0 to 1.0)
    pheromones: HashMap<(usize, usize, PheromoneType), f32>,
    grid_size: f32,
    width: usize,
    height: usize,
}
```

The system handles:
- Adding pheromones to specific locations
- Retrieving pheromone strength at any position
- Updating pheromones (evaporation) over time
- Rendering pheromones with appropriate colors (green for food, magenta for home)

## Ant Movement Based on Pheromones

### Pheromone Following Logic

Ants detect and follow pheromones in a sophisticated way that creates realistic foraging behavior:

1. **Different behaviors based on state**
   - When carrying food: Follow HOME pheromones to return to nest
   - When not carrying food: Follow FOOD pheromones to find food sources

2. **Direction sampling and sensing**
   - Ants sample pheromones in multiple directions around them
   - Carrying ants sample 6 directions (more focused forward)
   - Foraging ants sample 12 directions (wider exploration)
   - Each direction is sampled at multiple distances (5 sampling points)

3. **Forward bias for efficient travel**
   - Carrying ants have a strong forward bias to maintain momentum
   - Their direction sampling is concentrated in a forward arc
   - They ignore directions that would make them turn back significantly

4. **Gradual turning for natural movement**
   - Ants gradually turn toward the strongest pheromone direction
   - Turn rate is slower for carrying ants (0.1) for stable movement
   - Standard turn rate for foraging ants is higher (0.7) for better exploration

5. **Random variation to prevent perfect following**
   - Small random variations added to prevent circular trails
   - Carrying ants: 0.02 radian variation (subtle)
   - Foraging ants: 0.2 radian variation (more exploratory)

### Key Method Implementation

The `follow_pheromones` method determines which pheromones to follow and adjusts the ant's direction:

```rust
fn follow_pheromones(&mut self, environment: &Environment) {
    // Determine which pheromone to follow based on current state
    let pheromone_type = if self.carrying_food {
        PheromoneType::Home  // Follow home trails when carrying food
    } else {
        PheromoneType::Food  // Follow food trails when searching
    };
    
    // Find the direction with the strongest pheromone
    let best_direction = self.find_strongest_pheromone_direction(
        environment, 
        pheromone_type, 
        if self.carrying_food { 6 } else { 12 }
    );
    
    // If a direction with pheromones is found, adjust direction toward it
    if let Some(best_dir) = best_direction {
        // Calculate angle difference and gradually turn
        let angle_diff = (best_dir - self.direction + std::f32::consts::PI * 3.0) 
                         % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
        
        // Only turn if the pheromone is roughly ahead
        let forward_angle_limit = if self.carrying_food {
            std::f32::consts::PI * 0.5  // More forward-focused
        } else {
            std::f32::consts::PI * 2.0/3.0  // Wider angle when searching
        };
        
        if angle_diff.abs() < forward_angle_limit {
            // Turn rate varies by ant state
            let turn_rate = if self.carrying_food { 0.1 } else { 0.7 };
            self.direction += angle_diff * turn_rate;
        }
        
        // Add small random variation
        let random_variation = if self.carrying_food {
            (rand::random::<f32>() - 0.5) * 0.02
        } else {
            (rand::random::<f32>() - 0.5) * 0.2
        };
        self.direction += random_variation;
    } else {
        // No pheromones found, increase random movement
        // Higher chance of direction change when carrying food
    }
}
```

The `find_strongest_pheromone_direction` method samples the environment in multiple directions:

```rust
fn find_strongest_pheromone_direction(
    &self,
    environment: &Environment,
    pheromone_type: PheromoneType,
    num_directions: usize
) -> Option<f32> {
    let sense_distance = 40.0;
    let min_sense_distance = 5.0;
    
    // Sample points at different distances
    let sample_points = [min_sense_distance, 
                          sense_distance * 0.25, 
                          sense_distance * 0.5, 
                          sense_distance * 0.75, 
                          sense_distance];
    
    // Check each direction and find strongest pheromone
    // Returns the direction with strongest pheromone if above threshold
}
```

## Pheromone Deposit Logic

Ants deposit pheromones based on their state:

1. **Food pheromones** are deposited by ants returning to the nest with food
2. **Home pheromones** are deposited by ants leaving the nest to search for food
3. **Deposit strength varies** - stronger deposits at food sources and nest locations
4. **Pheromones accumulate** - multiple ants following the same path create stronger trails
5. **Temporary pheromone ignoring** - after finding food, ants temporarily ignore pheromones (15 seconds) to prevent getting stuck in circles

## Evaporation and Trail Maintenance

Pheromones gradually evaporate over time, implemented in the `update` method:

```rust
pub fn update(&mut self, delta_time: f32) {
    // Evaporation rate controls how quickly pheromones fade
    // Current rate: 0.005 * delta_time (0.5% per second)
    let evaporation_rate = 0.005 * delta_time;
    
    // Reduce strength of each pheromone
    // Remove pheromones below threshold (0.001)
}
```

This evaporation is crucial for:
1. Allowing trails to fade when no longer maintained
2. Creating dynamic paths that adapt to changing conditions
3. Preventing the environment from becoming oversaturated with old pheromones

## Visualization

Pheromones are rendered with specific colors to visualize the system:
- **Food pheromones**: Green (RGB: 0, 255, 100)
- **Home pheromones**: Magenta (RGB: 255, 50, 255)

The alpha channel represents pheromone strength, making stronger pheromones more visible.

## Emergent Behavior

The pheromone system creates several emergent behaviors:

1. **Path optimization** - Over time, ants find and reinforce shorter paths
2. **Adaptive foraging** - New food sources are quickly exploited
3. **Dynamic trail networks** - Trails form, strengthen, and fade based on usage
4. **Collective intelligence** - Individual simple rules lead to complex colony behavior 