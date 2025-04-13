use std::collections::HashMap;
use super::component::ComponentType;

/// Unique identifier for entities
pub type EntityId = u64;

/// Entity struct representing a game object
pub struct Entity {
    id: EntityId,
    pub components: HashMap<ComponentType, usize>,
}

impl Entity {
    /// Create a new entity with a unique ID
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            components: HashMap::new(),
        }
    }
    
    /// Get the entity's unique ID
    pub fn id(&self) -> EntityId {
        self.id
    }
    
    /// Add a component to this entity
    pub fn add_component(&mut self, component_type: ComponentType, component_index: usize) {
        self.components.insert(component_type, component_index);
    }
    
    /// Remove a component from this entity
    pub fn remove_component(&mut self, component_type: &ComponentType) {
        self.components.remove(component_type);
    }
    
    /// Check if entity has a specific component
    pub fn has_component(&self, component_type: &ComponentType) -> bool {
        self.components.contains_key(component_type)
    }
    
    /// Get the component index for a specific component type
    pub fn get_component_index(&self, component_type: &ComponentType) -> Option<usize> {
        self.components.get(component_type).copied()
    }
} 