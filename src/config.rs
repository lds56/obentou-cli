use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use toml;

use serde_json::{Map, Value};

use ratatui::style::Color;
use std::fs::OpenOptions;
use std::io::Write;

use crate::write_info;

use crate::data::MetaData;

// Define a struct to hold the configuration data
pub struct Config {
    metadata: MetaData,
}

impl Config {
    pub fn load(filename: &str) -> Result<Config> {
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

        let chosen_theme = themes.get("mondrian").context("No such a theme")?.clone();
        let theme: HashMap<String, Color> = cards.clone().into_iter()
                                                 .zip(chosen_theme.into_iter())
                                                 .collect();

        /*let metadata = MetaData {
            cards,
            shapes,
            fields,
        };*/

        Ok(Config {
            metadata: MetaData::new(
                cards,
                shapes,
                fields,
                theme,
            )
        })

    }

    pub fn get_metadata(&self) -> &MetaData {
        &self.metadata
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
