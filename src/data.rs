use std::collections::HashMap;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};

use ratatui::style::Color;
use serde_json::{Value,  json};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct MetaData {
    cards: Vec<String>, // card types: Note, Photo, ...
    shapes: Vec<String>, // card shapes: 1x4, 2x4, 4x4, ...
    fields: HashMap<String, Vec<String>>,
    theme: HashMap<String, Color>,
}

#[derive(Debug, Clone)]
pub struct Item {
    title: String,
    shape: String,
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub metadata: MetaData,
    pub items: Vec<Item>,
}

pub fn parse_data_from_file(filename: &str) -> Result<Vec<Item>> {

    // write_info!(format!("read file: {}", filename));

    let mut file = File::open(filename)?;

    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Failed to read file");

    let data_json: Value = serde_json::from_str(&data)?;

    Ok(parse_data(&data_json))
}

pub fn save_data_to_file(data: &Data, filename: &str) -> Result<()> {

    let mut showcase = json!([]);

    let profile_str = data.items.get(0).context("No profile found")?.get_lines().join("\n");
    let profile: Value = serde_json::from_str(&profile_str)?;

    for i in 1..data.items.len() {

        let item = data.items.get(i).context("No item found!")?;
        let content = item.get_lines().join("\n");

        let json_value: Result<Value, _> = serde_json::from_str(&content);
        match json_value {
            Ok(value) => {
                let item = json!({item.get_title(): value});
                showcase.as_array_mut().unwrap().push(item);
            }
            Err(_) => return Err(anyhow!("Parse json error!")),
        }
    }

    let output_data = json!({
        "profile": profile,
        "showcase": showcase,
    });

    // write_info!(format!("output: {}", output_data));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)?;

    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer_pretty(&mut file, &output_data)?;

    Ok(())
}

pub fn format_json(input: &str) -> Vec<String> {
    match serde_json::from_str::<Value>(input) {
        Ok(v) => {
            // 尝试重新格式化为漂亮的 JSON 字符串
            match serde_json::to_string_pretty(&v) {
                Ok(formatted_json) => formatted_json.split("\n").map(|s| s.to_string()).collect(),
                Err(_) => vec![input.to_string()], // 格式化失败，返回原始 JSON 字符串作为数组元素
            }
        }
        Err(_) => vec![input.to_string()], // 解析失败，返回原始 JSON 字符串作为数组元素
    }
}


fn format_json_value(value: &Value) -> Vec<String> {
    match serde_json::to_string_pretty(&value) {
        Ok(formatted_json) => formatted_json.split("\n").map(|s| s.to_string()).collect(),
        Err(_) => vec![value.to_string()], // 格式化失败，返回原始 JSON 字符串作为数组元素
    }
}


fn parse_data(json_data: &Value) -> Vec<Item> {

    let profile = &json_data["profile"];
    let showcase = &json_data["showcase"];

    let arr = showcase.as_array().unwrap();

    // .map(|v| serde_json::to_string(v).unwrap())
    //
    let mut items = vec![];
    items.push(Item {
        title: "Profile".to_string(),
        lines: format_json_value(&profile),
        shape: "4x4".to_string(),
    });

    // let mut cards = vec!["Profile".to_string()];
    // let mut contents = vec![format_json_value(&profile)];

    // Iterate over each object in the array
    for obj in arr {
        if let Value::Object(map) = obj {
            // Iterate over key-value pairs in the object
            for (key, value) in map {
                // if let value_map = serde_json::from_str(value) {
                let shape = if key != "Section"  {
                    if let Some(shape) = value.get("shape").and_then(Value::as_str) {
                        shape
                    } else {
                        "2x2"
                    }
                } else {
                    "1x8"
                };
                items.push( Item {
                    title: key.to_string(),
                    lines: format_json_value(value),
                    shape: shape.to_string(),
                } );
                // }
            }
        }
    }

    items
}


impl MetaData {

    pub fn new(cards: Vec<String>,
               shapes: Vec<String>,
               fields: HashMap<String, Vec<String>>,
               theme: HashMap<String, Color>) -> MetaData {
        MetaData {
            cards, shapes, fields, theme
        }
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

    pub fn get_card(&self, idx: usize) -> Option<&String> {
        self.cards.get(idx)
    }

    pub fn get_shape(&self, idx: usize) -> Option<&String> {
        self.shapes.get(idx)
    }

    pub fn get_field(&self, card_type: &str) -> Option<&Vec<String>> {
        self.fields.get(card_type)
    }

    pub fn count_shapes(&self) -> usize {
        self.shapes.len()
    }

    pub fn count_cards(&self) -> usize {
        self.cards.len()
    }

    pub fn index_of_shape(&self, shape: &str) -> usize {
        if let Some(index) = self.shapes.iter().position(|x| *x == shape) {
            index
        } else {
            0
        }
    }


    pub fn get_theme(&self) -> &HashMap<String, Color> {
        &self.theme
    }


    pub fn get_card_color(&self, card: &str) -> &Color {
        if let Some(color) = self.theme.get(card) {
            color
        } else {
            &Color::Gray
        }
    }

    /*
    pub fn get_card_color_by_index(&self, card_index: usize) -> &Color {
        if let Some(card) = self.cards.get(card_index) {
            if let Some(color) = self.theme.get(card) {
                return color;
            }

}
        &Color::Gray
    }
    */

    pub fn create_item(&self, card_index: usize, shape_index: usize) -> Result<Item> {

        let shape = if card_index == 0 {
            "1x8"
        } else {
            self.get_shape(shape_index).context("Unexpected shape index")?
        };

        let card = self.get_card(card_index).context("Unexpected card index")?;
        let fields = &self.fields[card];
        let mut strs = vec!["{".to_string()];
        for (index, field) in fields.iter().enumerate() {
            strs.push(format!("    \"{}\": \"\"{}",
                              if field.ends_with('?') { field.strip_suffix("?").unwrap() } else { field },
                              if index == fields.len() - 1 { "" } else { "," })
            );
        }
        strs.push("}".to_string());
        // for s in &strs {
           // write_info!(s);
        // }
        Ok(Item {
            title: card.to_string(),
            shape: shape.to_string(),
            lines: strs,
        })
    }

    pub fn is_valid(&self, json_str: &str, card_type: &str) -> Result<()> {
        match serde_json::from_str::<Value>(json_str) {
            Ok(v) => {
                let keys = self.get_field(card_type).context("No such field")?;
                for key in keys {
                    if !key.ends_with('?') {
                        if let Some(_) = v.get(key) {
                            continue;
                        } else {
                            return Err(anyhow!("Missing neccessary field!"));
                        }
                    }
                }
                Ok(())
            }
            Err(_) => Err(anyhow!("Invalid json format!"))
        }
    }

}

impl Item {

    pub fn new(title: String,
               shape: String,
               lines: Vec<String>) -> Item {

        Item { title, shape, lines }
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_shape(&self) -> &String {
        &self.shape
    }

    pub fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }

    pub fn get_line(&self, idx: usize) -> Option<&String> {
        self.lines.get(idx)
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_shape(&mut self, shape: String) {
        self.shape = shape;
    }

    pub fn set_lines(&mut self, lines: &Vec<String>) {
        self.lines = lines.to_vec();
    }

    pub fn set_lines_and_format(&mut self, lines: &[String]) {
        self.lines = format_json(lines.join("\n").as_str());
    }

}

/*
impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        writeln!(f, "  Cards: {:?}", self.cards)?;
        writeln!(f, "  Contents: ...")?;
        for content in &self.contents {
            writeln!(f, "    {}", content.join("\n"))?;
        }
        Ok(())
    }
}
*/
