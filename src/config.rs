use anyhow::{Context, Result};
use std::collections::HashMap;
use toml;

use serde_json::{Map, Value};

use ratatui::style::Color;
use std::fs::OpenOptions;
use std::io::Write;

use crate::write_info;

// Define a struct to hold the configuration data
pub struct Config {
    cards: Vec<String>,
    shapes: Vec<String>,
    fields: HashMap<String, Vec<String>>,
    themes: HashMap<String, Vec<Color>>,
}

impl Config {
    pub fn new(filename: &str) -> Result<Config> {
        let metadata = std::fs::read_to_string(filename)
            .with_context(|| "Failed to read config file")?;
        let parsed: toml::Value = toml::from_str(&metadata)
            .with_context(|| "Failed to parse config file")?;

        let cards: Vec<String> = parsed["Cards"]["types"]
            .as_array()
            .context("Invalid 'types' format")?
            .iter()
            .map(|s| s.as_str().context("Invalid string in 'types'").unwrap().to_string())
            .collect();

        let shapes: Vec<String> = parsed["Cards"]["shapes"]
            .as_array()
            .context("Invalid 'shapes' format")?
            .iter()
            .map(|s| s.as_str().context("Invalid string in 'shapes'").unwrap().to_string())
            .collect();

        let mut fields: HashMap<String, Vec<String>> = HashMap::new();
        for (key, value) in parsed["Cards"]["Fields"].as_table().context("Invalid 'Fields' format")? {
            let field_types: Vec<String> = value
                .as_array()
                .context(format!("Invalid format for field '{}'", key))?
                .iter()
                .map(|s| s.as_str().context("Invalid string in field types").unwrap().to_string())
                .collect();
            fields.insert(key.to_string(), field_types);
        }

        let mut themes: HashMap<String, Vec<Color>> = HashMap::new();
        for (key, value) in parsed["Themes"].as_table().context("Invalid 'Themes' format")? {
            let theme_colors: Vec<Color> = value
                .as_array()
                .context(format!("Invalid format for theme '{}'", key))?
                .iter()
                .map(|v| v.as_integer().context("Invalid integer in theme colors").unwrap() as u8)
                .map(|i| Color::Indexed(i))
                .collect();
            themes.insert(key.to_string(), theme_colors);
        }

        Ok(Config {
            cards,
            shapes,
            fields,
            themes,
        })
    }


    pub fn get_cards(&self) -> &Vec<String> {
        &self.cards
    }

    pub fn get_shapes(&self) -> &Vec<String> {
        &self.shapes
    }

    pub fn get_fields(&self) -> &HashMap<String, Vec<String>> {
        &self.fields
    }

    pub fn get_themes(&self) -> &HashMap<String, Vec<Color>> {
        &self.themes
    }

    
    pub fn get_chosen_theme(&self) -> &Vec<Color> {
        self.themes.get("mondrian").unwrap()
            // .unwrap_or(&vec![Color::Gray; 10])
            // .to_vec()
}

    pub fn get_shape(&self, idx: usize) -> &str {
        self.shapes.get(idx).unwrap() //_or(&"2x2".to_string()).to_string()
    }

    pub fn get_card(&self, idx: usize) -> &str {
        self.cards.get(idx).unwrap() // _or(&"Note".to_string()).to_string()
    }

    pub fn get_shapes_size(&self) -> usize {
        self.shapes.len()
    }

    pub fn get_cards_size(&self) -> usize {
        self.cards.len()
    }

    pub fn get_card_color(&self, card: &str) -> Color {
        if let Some(index) = self.cards.iter().position(|x| *x == card) {
            self.get_chosen_theme()[index]
        }
        else {
            Color::Gray
        }
    }

    pub fn create_card(&self, card_index: usize) -> Vec<String> {
        let card = &self.cards[card_index];
        let fields = &self.fields[card];
        let mut strs = vec!["{".to_string()];
        for (index, field) in fields.iter().enumerate() {
            strs.push(format!("    \"{}\": \"\"{}", field,
                              if index == fields.len() - 1 { "" } else { "," })
            );
        }
        strs.push("}".to_string());
        // for s in &strs {
           // write_info!(s);
        // }
        strs
    }
}
// fn main() -> Result<()> {
// &    le.to_string()t config = Config::new()?;
//
//     println!("Types: {:?}", config.types);
//     println!("Shapes: {:?}", config.shapes);
//     println!("Fields: {:?}", config.fields);
//     println!("Themes: {:?}", config.themes);
//
//     Ok(())
// }
