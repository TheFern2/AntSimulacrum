use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Text, Font, Color, Transformable, Shape};
use sfml::system::Vector2f;
use sfml::window::{Event, mouse};
use sfml::cpp::FBox;

use crate::game::InteractionMode;
use crate::environment::Environment;

// UI layout constants
const STATUS_BAR_HEIGHT: f32 = 30.0;

pub struct UI {
    width: u32,
    height: u32,
    status_text: Text<'static>,
    control_text: Text<'static>,
    food_delivery_text: Text<'static>,
    font: Box<FBox<Font>>,
    current_mode: InteractionMode,
}

impl UI {
    pub fn new(width: u32, height: u32) -> Self {
        // Load font - we'll use a system font as a fallback
        let font_result = Font::from_file("/System/Library/Fonts/Supplemental/Arial.ttf")
            .or_else(|_| Font::from_file("/usr/share/fonts/TTF/DejaVuSans.ttf"));
            
        let font = match font_result {
            Ok(f) => Box::new(f),
            Err(_) => panic!("Failed to load any font")
        };
        
        // Since SFML 0.24.0 has changed lifetime handling, we need to ensure
        // our text objects have static lifetimes. We'll leak the font reference
        // since it will live for the duration of the program anyway.
        let font_ref = unsafe { 
            let raw_ptr = &**font as *const Font;
            &*raw_ptr 
        };
        
        let mut status_text = Text::new("", font_ref, 14);
        status_text.set_position(Vector2f::new(10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        status_text.set_fill_color(Color::BLACK);

        let mut control_text = Text::new(
            "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [S]ave | [L]oad | [ESC]lear | [SPACE]Pause", 
            font_ref, 
            14
        );
        control_text.set_position(Vector2f::new(width as f32 - control_text.global_bounds().width - 10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        control_text.set_fill_color(Color::BLACK);
        
        // Create food delivery counter text
        let mut food_delivery_text = Text::new("Food Deliveries: 0", font_ref, 16);
        food_delivery_text.set_position(Vector2f::new(10.0, 10.0));
        food_delivery_text.set_fill_color(Color::rgb(200, 100, 0)); // Orange
        
        Self {
            width,
            height,
            status_text,
            control_text,
            food_delivery_text,
            font,
            current_mode: InteractionMode::None,
        }
    }
    
    pub fn handle_event(&mut self, _event: &Event) {
        // No UI elements to interact with now
    }
    
    pub fn update(&mut self, interaction_mode: &InteractionMode, simulation_speed: f32, paused: bool, environment: &Environment) {
        // Save current mode for rendering
        self.current_mode = interaction_mode.clone();
        
        // Update status text - ONLY show game status, not controls
        let status = format!(
            "Mode: {:?} | Speed: {:.1}x | {}",
            interaction_mode,
            simulation_speed,
            if paused { "PAUSED" } else { "Running" }
        );
        self.status_text.set_string(&status);
        
        // Update food delivery counter
        let total_deliveries: u32 = environment.get_all_colonies().iter()
            .map(|colony| colony.get_food_deliveries())
            .sum();
        
        println!("DEBUG: UI updating food counter - Total deliveries: {}", total_deliveries);
        
        // Log each colony's individual count
        for (i, colony) in environment.get_all_colonies().iter().enumerate() {
            println!("DEBUG: Colony #{} has {} food deliveries", i, colony.get_food_deliveries());
        }
        
        self.food_delivery_text.set_string(&format!("Food Deliveries: {}", total_deliveries));
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        // Update positions of UI elements
        self.status_text.set_position(Vector2f::new(10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        self.control_text.set_position(Vector2f::new(width as f32 - self.control_text.global_bounds().width - 10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        // Food counter stays at the top-left corner
        self.food_delivery_text.set_position(Vector2f::new(10.0, 10.0));
    }
    
    pub fn render(&mut self, window: &mut RenderWindow) {
        // Draw status bar background
        let mut status_bg = RectangleShape::new();
        status_bg.set_size(Vector2f::new(self.width as f32, STATUS_BAR_HEIGHT));
        status_bg.set_position(Vector2f::new(0.0, self.height as f32 - STATUS_BAR_HEIGHT));
        status_bg.set_fill_color(Color::rgb(230, 230, 230));
        window.draw(&status_bg);
        
        // Draw status text
        window.draw(&self.status_text);
        
        // Draw food delivery counter
        window.draw(&self.food_delivery_text);
        
        // Get a font reference to create the text elements
        let font_ref = unsafe { 
            let raw_ptr = &**self.font as *const Font;
            &*raw_ptr 
        };
        
        // Define the parts of the control text
        let parts = [
            ("[W]all", InteractionMode::AddWall),
            ("[F]ood", InteractionMode::AddFood),
            ("[R]emove", InteractionMode::RemoveObject),
            ("[N]est", InteractionMode::AddAntNest),
            ("[A]nt", InteractionMode::AddAnt),
            ("[S]ave", InteractionMode::None),
            ("[L]oad", InteractionMode::None),
            ("[ESC]lear", InteractionMode::None),
            ("[SPACE]Pause", InteractionMode::None)
        ];

        // Calculate starting x position - we'll position at the right side of the window
        // First estimate the total width 
        let total_width_estimate = 400.0; // Rough estimate to ensure it's not too tight against the right edge
        let base_x = self.width as f32 - total_width_estimate - 10.0;
        let base_y = self.height as f32 - STATUS_BAR_HEIGHT + 5.0;
        let mut current_x = base_x;
        
        // Draw each part of the control text with appropriate styling
        for (i, (text_part, mode)) in parts.iter().enumerate() {
            let mut part_text = Text::new(*text_part, font_ref, 14);
            part_text.set_fill_color(Color::BLACK);
            
            // Add underline style if this is the active mode
            if !matches!(self.current_mode, InteractionMode::None) && 
               std::mem::discriminant(&self.current_mode) == std::mem::discriminant(mode) {
                part_text.set_style(sfml::graphics::TextStyle::UNDERLINED);
            }
            
            part_text.set_position(Vector2f::new(current_x, base_y));
            window.draw(&part_text);
            
            // Add separator between controls except for the last one
            if i < parts.len() - 1 {
                let separator = " | ";
                let mut sep_text = Text::new(separator, font_ref, 14);
                sep_text.set_fill_color(Color::BLACK);
                sep_text.set_position(Vector2f::new(current_x + part_text.global_bounds().width, base_y));
                window.draw(&sep_text);
                current_x += part_text.global_bounds().width + sep_text.global_bounds().width;
            } else {
                current_x += part_text.global_bounds().width;
            }
        }
    }
} 