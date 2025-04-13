use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Text, Font, Color, Transformable, Shape};
use sfml::system::Vector2f;
use sfml::window::{Event, mouse};
use sfml::cpp::FBox;

use crate::game::InteractionMode;

// UI layout constants
const BUTTON_WIDTH: f32 = 80.0;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_SPACING: f32 = 10.0;
const BUTTON_Y_POSITION: f32 = 10.0;

struct Button {
    rect: RectangleShape<'static>,
    text: Text<'static>,
    action: InteractionMode,
}

pub struct UI {
    width: u32,
    height: u32,
    buttons: Vec<Button>,
    status_text: Text<'static>,
    font: Box<FBox<Font>>,
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
        status_text.set_position(Vector2f::new(10.0, height as f32 - 30.0));
        status_text.set_fill_color(Color::BLACK);
        
        let mut ui = Self {
            width,
            height,
            buttons: Vec::new(),
            status_text,
            font,
        };
        
        // Create toolbar buttons
        ui.create_button("None", InteractionMode::None, 0);
        ui.create_button("Wall", InteractionMode::AddWall, 1);
        ui.create_button("Food", InteractionMode::AddFood, 2);
        ui.create_button("Remove", InteractionMode::RemoveObject, 3);
        ui.create_button("Nest", InteractionMode::AddAntNest, 4);
        
        ui
    }
    
    fn create_button(&mut self, label: &str, action: InteractionMode, position: usize) {
        // Get a static reference to the font (unsafe but necessary for static lifetime)
        let font_ref = unsafe { 
            let raw_ptr = &**self.font as *const Font;
            &*raw_ptr 
        };
        
        let mut rect = RectangleShape::new();
        rect.set_size(Vector2f::new(BUTTON_WIDTH, BUTTON_HEIGHT));
        rect.set_position(Vector2f::new(
            10.0 + (BUTTON_WIDTH + BUTTON_SPACING) * position as f32,
            BUTTON_Y_POSITION
        ));
        rect.set_fill_color(Color::rgb(200, 200, 200));
        rect.set_outline_thickness(1.0);
        rect.set_outline_color(Color::rgb(100, 100, 100));
        
        let mut text = Text::new(label, font_ref, 14);
        text.set_position(Vector2f::new(
            rect.position().x + 10.0,
            rect.position().y + 5.0
        ));
        text.set_fill_color(Color::BLACK);
        
        self.buttons.push(Button { 
            rect, 
            text,
            action,
        });
    }
    
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::MouseButtonPressed { button: mouse::Button::Left, x, y } => {
                // Check if any button was clicked
                for button in &self.buttons {
                    let bounds = button.rect.global_bounds();
                    if bounds.contains(Vector2f::new(*x as f32, *y as f32)) {
                        // Button clicked - will be processed in the update method
                        return;
                    }
                }
            }
            _ => {}
        }
    }
    
    pub fn update(&mut self, current_mode: &InteractionMode, simulation_speed: f32, paused: bool) {
        // Update button states based on current mode
        for button in &mut self.buttons {
            if std::mem::discriminant(&button.action) == std::mem::discriminant(current_mode) {
                button.rect.set_fill_color(Color::rgb(150, 150, 255));
            } else {
                button.rect.set_fill_color(Color::rgb(200, 200, 200));
            }
        }
        
        // Update status text
        let mode_text = match current_mode {
            InteractionMode::None => "Browse",
            InteractionMode::AddWall => "Add Wall",
            InteractionMode::AddFood => "Add Food",
            InteractionMode::RemoveObject => "Remove Objects",
            InteractionMode::AddAntNest => "Add Ant Nest",
        };
        
        let status = format!(
            "Mode: {} | Speed: {:.1}x | {}",
            mode_text,
            simulation_speed,
            if paused { "PAUSED" } else { "Running" }
        );
        
        self.status_text.set_string(&status);
    }
    
    pub fn render(&self, window: &mut RenderWindow) {
        // Draw toolbar background
        let mut toolbar_bg = RectangleShape::new();
        toolbar_bg.set_size(Vector2f::new(self.width as f32, BUTTON_HEIGHT + 20.0));
        toolbar_bg.set_position(Vector2f::new(0.0, 0.0));
        toolbar_bg.set_fill_color(Color::rgb(230, 230, 230));
        window.draw(&toolbar_bg);
        
        // Draw status bar background
        let mut status_bg = RectangleShape::new();
        status_bg.set_size(Vector2f::new(self.width as f32, 30.0));
        status_bg.set_position(Vector2f::new(0.0, self.height as f32 - 30.0));
        status_bg.set_fill_color(Color::rgb(230, 230, 230));
        window.draw(&status_bg);
        
        // Draw buttons
        for button in &self.buttons {
            window.draw(&button.rect);
            window.draw(&button.text);
        }
        
        // Draw status text
        window.draw(&self.status_text);
    }
} 