use sfml::graphics::{RenderWindow, RenderTarget, Color};
use sfml::window::{Style, Event, Key, mouse};
use sfml::system::Vector2f;
use sfml::cpp::FBox;
use std::path::Path;

use crate::environment::Environment;
use crate::ui::UI;
use crate::ant::Ant;
use crate::save;
use rand::random;

#[derive(Clone, Debug)]
pub enum InteractionMode {
    None,
    AddWall,
    AddFood,
    RemoveObject,
    AddAntNest,
    AddAnt,
}

pub struct Game {
    window: FBox<RenderWindow>,
    environment: Environment,
    ui: UI,
    interaction_mode: InteractionMode,
    simulation_speed: f32,
    paused: bool,
    left_mouse_pressed: bool,
    test_ants: Vec<Ant>, // Just for testing, will move to ECS later
    save_path: String,
}

impl Game {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let window = match RenderWindow::new(
            (width, height),
            title,
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        ) {
            Ok(window) => window,
            Err(e) => panic!("Failed to create render window: {:?}", e),
        };
        
        let environment = Environment::new(width, height);
        let ui = UI::new(width, height);
        
        // Create some test ants
        let mut test_ants = Vec::new();
        for _ in 0..50 {
            test_ants.push(Ant::new(
                random::<f32>() * width as f32,
                random::<f32>() * height as f32
            ));
        }
        
        Self {
            window,
            environment,
            ui,
            interaction_mode: InteractionMode::None,
            simulation_speed: 1.0,
            paused: false,
            left_mouse_pressed: false,
            test_ants,
            save_path: "ant_simulation_save.json".to_string(),
        }
    }
    
    pub fn run(&mut self) {
        self.window.set_vertical_sync_enabled(true);
        
        while self.window.is_open() {
            self.handle_events();
            self.update();
            self.render();
        }
    }
    
    fn handle_events(&mut self) {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => self.window.close(),
                Event::KeyPressed { code, .. } => self.handle_key_press(code),
                Event::MouseButtonPressed { button, x, y } => {
                    if button == mouse::Button::Left {
                        self.left_mouse_pressed = true;
                        self.handle_mouse_press(x, y);
                    }
                }
                Event::MouseButtonReleased { button, .. } => {
                    if button == mouse::Button::Left {
                        self.left_mouse_pressed = false;
                    }
                }
                Event::MouseMoved { x, y } => {
                    self.handle_mouse_move(x, y);
                }
                Event::Resized { width, height } => {
                    // Get the current view
                    let mut view = self.window.view().to_owned();
                    
                    // Update the view size
                    view.set_size((width as f32, height as f32));
                    view.set_center((width as f32 / 2.0, height as f32 / 2.0));
                    
                    // Apply the view
                    self.window.set_view(&view);
                    
                    // Inform UI of the resize
                    self.ui.resize(width, height);
                    
                    // Update environment to handle the new window size
                    self.environment.resize(width, height);
                }
                _ => {}
            }
            
            // Pass events to UI
            self.ui.handle_event(&event);
        }
    }
    
    fn handle_key_press(&mut self, key: Key) {
        match key {
            Key::Space => self.paused = !self.paused,
            Key::Add | Key::Equal => self.simulation_speed *= 1.2,
            Key::Subtract | Key::Hyphen => self.simulation_speed *= 0.8,
            Key::W => self.interaction_mode = InteractionMode::AddWall,
            Key::F => self.interaction_mode = InteractionMode::AddFood,
            Key::R => self.interaction_mode = InteractionMode::RemoveObject,
            Key::N => self.interaction_mode = InteractionMode::AddAntNest,
            Key::A => self.interaction_mode = InteractionMode::AddAnt,
            Key::Escape => self.interaction_mode = InteractionMode::None,
            Key::S => {
                // Save the game state
                if let Err(e) = self.save_game() {
                    println!("Error saving game: {}", e);
                } else {
                    println!("Game saved successfully to {}", self.save_path);
                }
            },
            Key::L => {
                // Load the game state
                if let Err(e) = self.load_game() {
                    println!("Error loading game: {}", e);
                } else {
                    println!("Game loaded successfully from {}", self.save_path);
                }
            },
            _ => {}
        }
    }
    
    fn handle_mouse_press(&mut self, x: i32, y: i32) {
        match self.interaction_mode {
            InteractionMode::AddWall => {
                self.environment.add_wall(x as f32, y as f32);
            }
            InteractionMode::AddFood => {
                self.environment.add_food(x as f32, y as f32);
            }
            InteractionMode::RemoveObject => {
                self.environment.remove_object(x as f32, y as f32);
            }
            InteractionMode::AddAntNest => {
                self.environment.add_ant_nest(x as f32, y as f32);
            }
            InteractionMode::AddAnt => {
                self.test_ants.push(Ant::new(x as f32, y as f32));
            }
            _ => {}
        }
    }
    
    fn handle_mouse_move(&mut self, x: i32, y: i32) {
        // Handle drag interactions
        if self.left_mouse_pressed {
            match self.interaction_mode {
                InteractionMode::AddWall => {
                    self.environment.add_wall(x as f32, y as f32);
                }
                InteractionMode::AddFood => {
                    self.environment.add_food(x as f32, y as f32);
                }
                InteractionMode::RemoveObject => {
                    self.environment.remove_object(x as f32, y as f32);
                }
                _ => {}
            }
        }
    }
    
    fn update(&mut self) {
        if !self.paused {
            let delta_time = self.simulation_speed / 60.0; // Assuming 60 FPS
            
            // Update environment
            self.environment.update(delta_time);
            
            // Update test ants
            for ant in &mut self.test_ants {
                ant.update(delta_time, &mut self.environment);
            }
        }
        
        self.ui.update(&self.interaction_mode, self.simulation_speed, self.paused);
    }
    
    fn render(&mut self) {
        self.window.clear(Color::rgb(240, 230, 210)); // Light sandy color
        
        self.environment.render(&mut self.window);
        
        // Render test ants
        for ant in &self.test_ants {
            ant.render(&mut self.window);
        }
        
        self.ui.render(&mut self.window);
        
        self.window.display();
    }
    
    // Save the current game state
    pub fn save_game(&self) -> std::io::Result<()> {
        save::save_game_state(Path::new(&self.save_path), self)
    }
    
    // Load a game state
    pub fn load_game(&mut self) -> std::io::Result<()> {
        let saved_game = save::load_game_state(Path::new(&self.save_path))?;
        
        // Load environment
        self.environment = saved_game.environment.to_environment();
        
        // Load test ants
        self.test_ants.clear();
        for saved_ant in saved_game.test_ants {
            self.test_ants.push(saved_ant.to_ant());
        }
        
        // Parse interaction mode from string
        match saved_game.interaction_mode.as_str() {
            "None" => self.interaction_mode = InteractionMode::None,
            "AddWall" => self.interaction_mode = InteractionMode::AddWall,
            "AddFood" => self.interaction_mode = InteractionMode::AddFood,
            "RemoveObject" => self.interaction_mode = InteractionMode::RemoveObject,
            "AddAntNest" => self.interaction_mode = InteractionMode::AddAntNest,
            "AddAnt" => self.interaction_mode = InteractionMode::AddAnt,
            _ => self.interaction_mode = InteractionMode::None,
        }
        
        // Load other game settings
        self.simulation_speed = saved_game.simulation_speed;
        self.paused = saved_game.paused;
        
        Ok(())
    }
    
    // Accessor methods for save/load functionality
    pub fn get_environment(&self) -> &Environment {
        &self.environment
    }
    
    pub fn get_interaction_mode(&self) -> &InteractionMode {
        &self.interaction_mode
    }
    
    pub fn get_simulation_speed(&self) -> f32 {
        self.simulation_speed
    }
    
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    pub fn get_test_ants(&self) -> &Vec<Ant> {
        &self.test_ants
    }
} 