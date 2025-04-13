use std::any::Any;
use std::collections::{HashMap, HashSet};

use super::component::{Component, ComponentType};
use super::entity::{Entity, EntityId};
use super::system::System;

/// Main world struct that manages all entities, components and systems
pub struct World {
    // Entity management
    entities: HashMap<EntityId, Entity>,
    next_entity_id: EntityId,
    
    // Component storage - each component type has a vector of components
    components: HashMap<ComponentType, Vec<Box<dyn Component>>>,
    
    // Systems for updating the world
    systems: Vec<Box<dyn System>>,
    
    // Resources - global data accessible to systems
    resources: HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_entity_id: 0,
            components: HashMap::new(),
            systems: Vec::new(),
            resources: HashMap::new(),
        }
    }
    
    /// Add a resource to the world
    pub fn add_resource<T: 'static>(&mut self, resource: T) {
        let type_id = std::any::TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }
    
    /// Get a reference to a resource
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.resources.get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Get a mutable reference to a resource
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = std::any::TypeId::of::<T>();
        self.resources.get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Create a new entity and return its ID
    pub fn create_entity(&mut self) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity = Entity::new(entity_id);
        self.entities.insert(entity_id, entity);
        
        entity_id
    }
    
    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        // Remove all components for this entity
        if let Some(entity) = self.entities.get(&entity_id) {
            for (component_type, &component_index) in &entity.components {
                if let Some(components) = self.components.get_mut(component_type) {
                    // Mark component as removed - we don't actually remove it
                    // as that would invalidate indices for other entities
                    components[component_index] = create_empty_component(*component_type);
                }
            }
        }
        
        // Remove the entity
        self.entities.remove(&entity_id);
    }
    
    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, component: T) {
        let component_type = component.component_type();
        
        // Get or create component storage for this type
        let components = self.components
            .entry(component_type)
            .or_insert_with(Vec::new);
        
        // Add component to storage and get its index
        let component_index = components.len();
        components.push(Box::new(component));
        
        // Add component reference to entity
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.add_component(component_type, component_index);
        }
    }
    
    /// Remove a component from an entity
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: ComponentType) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.remove_component(&component_type);
        }
    }
    
    /// Get a component for an entity by type
    pub fn get_component<T: Component + 'static>(&self, entity_id: EntityId, component_type: ComponentType) -> Option<&T> {
        let entity = self.entities.get(&entity_id)?;
        let component_index = entity.get_component_index(&component_type)?;
        let components = self.components.get(&component_type)?;
        
        if component_index >= components.len() {
            return None;
        }
        
        components[component_index].as_any().downcast_ref::<T>()
    }
    
    /// Get a mutable component for an entity by type
    pub fn get_component_mut<T: Component + 'static>(&mut self, entity_id: EntityId, component_type: ComponentType) -> Option<&mut T> {
        let entity = self.entities.get(&entity_id)?;
        let component_index = entity.get_component_index(&component_type)?;
        let components = self.components.get_mut(&component_type)?;
        
        if component_index >= components.len() {
            return None;
        }
        
        components[component_index].as_any_mut().downcast_mut::<T>()
    }
    
    /// Get all entities that have all of the specified component types
    pub fn get_entities_with_components(&self, component_types: &[ComponentType]) -> Vec<EntityId> {
        let mut result = Vec::new();
        
        for (&entity_id, entity) in &self.entities {
            let has_all_components = component_types
                .iter()
                .all(|component_type| entity.has_component(component_type));
                
            if has_all_components {
                result.push(entity_id);
            }
        }
        
        result
    }
    
    /// Add a system to the world
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }
    
    /// Update all systems
    pub fn update(&mut self, delta_time: f32) {
        // Since we can't borrow self mutably multiple times,
        // we'll temporarily replace the systems with an empty vector,
        // update each system, then put them back
        let mut systems = std::mem::replace(&mut self.systems, Vec::new());
        
        for mut system in systems.drain(..) {
            system.update(self, delta_time);
            self.systems.push(system);
        }
    }
}

/// Create an empty component of the specified type
/// This is used when "removing" components to avoid invalidating indices
fn create_empty_component(component_type: ComponentType) -> Box<dyn Component> {
    match component_type {
        ComponentType::Position => Box::new(super::component::PositionComponent { x: 0.0, y: 0.0 }),
        ComponentType::Velocity => Box::new(super::component::VelocityComponent { 
            dx: 0.0, dy: 0.0, speed: 0.0, direction: 0.0 
        }),
        ComponentType::Appearance => Box::new(super::component::AppearanceComponent {
            shape_type: super::component::ShapeType::Circle,
            radius: 0.0,
            width: 0.0,
            height: 0.0,
            color: (0, 0, 0),
        }),
        ComponentType::AntState => Box::new(super::component::AntStateComponent {
            state: super::component::AntState::Idle,
            carrying_food: false,
            pheromone_timer: 0.0,
            random_direction_timer: 0.0,
            home_position: (0.0, 0.0),
            colony_id: None,
        }),
        // Add empty versions for other component types as needed
        _ => panic!("Unimplemented component type for empty creation"),
    }
} 