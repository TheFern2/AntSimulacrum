use std::any::Any;
use sfml::system::Vector2f;

/// Enum defining all possible component types
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ComponentType {
    Position,
    Velocity,
    Appearance,
    Health,
    Pheromone,
    AntState,
    Collider,
    Food,
    Colony,
}

/// Trait that all components must implement
pub trait Component: Any + Send + Sync {
    /// Get the type of this component
    fn component_type(&self) -> ComponentType;
    
    /// Clone as box - for storing in component vectors
    fn clone_boxed(&self) -> Box<dyn Component>;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Position component
#[derive(Clone, Debug)]
pub struct PositionComponent {
    pub x: f32,
    pub y: f32,
}

impl Component for PositionComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Position
    }
    
    fn clone_boxed(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Velocity component
#[derive(Clone, Debug)]
pub struct VelocityComponent {
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
    pub direction: f32, // in radians
}

impl Component for VelocityComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Velocity
    }
    
    fn clone_boxed(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Appearance component
#[derive(Clone, Debug)]
pub struct AppearanceComponent {
    pub shape_type: ShapeType,
    pub radius: f32,
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8), // RGB
}

#[derive(Clone, Debug)]
pub enum ShapeType {
    Circle,
    Rectangle,
}

impl Component for AppearanceComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Appearance
    }
    
    fn clone_boxed(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Ant state component for ant behavior
#[derive(Clone, Debug)]
pub struct AntStateComponent {
    pub state: AntState,
    pub carrying_food: bool,
    pub pheromone_timer: f32,
    pub random_direction_timer: f32,
    pub home_position: (f32, f32),
    pub colony_id: Option<super::entity::EntityId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AntState {
    Idle,
    SearchingForFood,
    ReturningHome,
    FollowingPheromone,
}

impl Component for AntStateComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::AntState
    }
    
    fn clone_boxed(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
} 