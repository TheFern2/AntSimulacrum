# Ant Simulacrum Architecture Overview

This document provides a high-level overview of the Ant Simulacrum architecture.

## Architecture Diagram

```mermaid
graph TD
    Main[main.rs] --> Game[game/mod.rs]
    
    %% Main Modules
    Game --> ECS[ecs/]
    Game --> UI[ui/]
    Game --> Environment[environment/mod.rs]
    Game --> Save[save/]
    
    %% ECS Structure
    ECS --> World[ecs/world.rs]
    ECS --> Entity[ecs/entity.rs]
    ECS --> Component[ecs/component.rs]
    ECS --> System[ecs/system.rs]
    
    %% Game Elements
    Game --> Colony[colony/]
    Game --> Ant[ant/mod.rs]
    Game --> Pheromone[pheromone/]
    
    %% Connections
    Environment --> Pheromone
    Colony --> Ant
    Ant --> Pheromone
    World --> Entity
    World --> Component
    World --> System
    
    %% External Dependencies
    ExternalDeps[External Dependencies] --> SFML[SFML Graphics]
    ExternalDeps --> Rand[Random]
    ExternalDeps --> Serde[Serialization/Deserialization]
    ExternalDeps --> Logger[Logging]
    
    Game --> ExternalDeps
```

## Module Descriptions

- **main.rs**: Program entry point, initializes the game and logging
- **game/**: Core game loop and state management
- **ecs/**: Entity Component System implementation
  - **world.rs**: Manages entities, components and systems
  - **entity.rs**: Entity representation
  - **component.rs**: Component types and storage
  - **system.rs**: Game logic systems
- **ant/**: Ant entities, behaviors and states
- **colony/**: Colony management and properties
- **environment/**: World environment elements like food and obstacles
- **pheromone/**: Pheromone system for ant communication
- **ui/**: User interface elements
- **save/**: Save/load game state functionality

## Data Flow

1. The game loop runs in the Game module
2. The ECS World manages entities (ants, food, etc.) and their components
3. Systems process entities with specific components each frame
4. Ants interact with the environment and deposit pheromones
5. The UI renders the current state and handles user input
6. Save system serializes/deserializes game state when requested 