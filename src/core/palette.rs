use crate::core::color::Color;
use rust_i18n::t;

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn new() -> Self {
        Self { name: t!("palette.default_custom").to_string(), colors: Vec::new() }
    }

    pub fn default_pico8() -> Self {
        Self {
            name: t!("palette.default_pico8").to_string(),
            colors: vec![
                Color::new(0, 0, 0, 255),       
                Color::new(29, 43, 83, 255),    
                Color::new(126, 37, 83, 255),   
                Color::new(0, 135, 81, 255),    
                Color::new(171, 82, 54, 255),   
                Color::new(95, 87, 79, 255),    
                Color::new(194, 195, 199, 255), 
                Color::new(255, 241, 232, 255), 
                Color::new(255, 0, 77, 255),    
                Color::new(255, 163, 0, 255),   
                Color::new(255, 236, 39, 255),  
                Color::new(0, 228, 54, 255),    
                Color::new(41, 173, 255, 255),  
                Color::new(131, 118, 156, 255), 
                Color::new(255, 119, 168, 255), 
                Color::new(255, 204, 170, 255), 
            ]
        }
    }

    pub fn add_color(&mut self, color: Color) {
        if !self.colors.contains(&color) {
            self.colors.push(color);
        }
    }

    pub fn remove_color(&mut self, index: usize) {
        if index < self.colors.len() {
            self.colors.remove(index);
        }
    }
}