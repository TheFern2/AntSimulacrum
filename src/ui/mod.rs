use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Text, Font, Color, Transformable, Shape};
use sfml::system::Vector2f;
use sfml::window::{Event, mouse};
use sfml::cpp::FBox;

use crate::game::InteractionMode;

// UI layout constants
const STATUS_BAR_HEIGHT: f32 = 30.0;

pub struct UI {
    width: u32,
    height: u32,
    status_text: Text<'static>,
    control_text: Text<'static>,
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
            "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause", 
            font_ref, 
            14
        );
        control_text.set_position(Vector2f::new(width as f32 - control_text.global_bounds().width - 10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        control_text.set_fill_color(Color::BLACK);
        
        Self {
            width,
            height,
            status_text,
            control_text,
            font,
            current_mode: InteractionMode::None,
        }
    }
    
    pub fn handle_event(&mut self, _event: &Event) {
        // No UI elements to interact with now
    }
    
    pub fn update(&mut self, current_mode: &InteractionMode, simulation_speed: f32, paused: bool) {
        // Save current mode for rendering
        self.current_mode = current_mode.clone();
        
        // Update status text
        let mode_text = match current_mode {
            InteractionMode::None => "Browse",
            InteractionMode::AddWall => "Add Wall",
            InteractionMode::AddFood => "Add Food",
            InteractionMode::RemoveObject => "Remove Objects",
            InteractionMode::AddAntNest => "Add Ant Nest",
            InteractionMode::AddAnt => "Add Ant",
        };
        
        let status = format!(
            "Mode: {} | Speed: {:.1}x | {}",
            mode_text,
            simulation_speed,
            if paused { "PAUSED" } else { "Running" }
        );
        
        self.status_text.set_string(&status);
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        // Update positions of UI elements
        self.status_text.set_position(Vector2f::new(10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
        self.control_text.set_position(Vector2f::new(width as f32 - self.control_text.global_bounds().width - 10.0, height as f32 - STATUS_BAR_HEIGHT + 5.0));
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
        
        // Get a font reference to create the underlined text
        let font_ref = unsafe { 
            let raw_ptr = &**self.font as *const Font;
            &*raw_ptr 
        };
        
        // Create an underlined version of the control text that shows which mode is active
        let controls = match self.current_mode {
            InteractionMode::None => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
            InteractionMode::AddWall => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
            InteractionMode::AddFood => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
            InteractionMode::RemoveObject => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
            InteractionMode::AddAntNest => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
            InteractionMode::AddAnt => 
                "[W]all | [F]ood | [R]emove | [N]est | [A]nt | [ESC]lear | [SPACE]Pause",
        };
        self.control_text.set_string(controls);
        
        // Create underlined version of the active control
        let mut highlight = Text::new("", font_ref, 14);
        highlight.set_style(sfml::graphics::TextStyle::UNDERLINED);
        highlight.set_fill_color(Color::BLACK);
        
        // Position and text based on active mode
        let (text, offset) = match self.current_mode {
            InteractionMode::AddWall => ("[W]all", 0),
            InteractionMode::AddFood => ("[F]ood", 8),
            InteractionMode::RemoveObject => ("[R]emove", 16),
            InteractionMode::AddAntNest => ("[N]est", 27),
            InteractionMode::AddAnt => ("[A]nt", 36),
            InteractionMode::None => ("", 0), // No underline for None mode
        };
        
        if !text.is_empty() {
            highlight.set_string(text);
            let base_x = self.control_text.position().x + offset as f32;
            highlight.set_position(Vector2f::new(base_x, self.height as f32 - STATUS_BAR_HEIGHT + 5.0));
            window.draw(&highlight);
        }
        
        // Draw control text
        window.draw(&self.control_text);
    }
} 