use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::fmt;

use serde_json::{Value,  json};

use anyhow::{anyhow, Context, Result};


#[derive(Debug, Default, Clone)]
pub struct Data {
    pub cards: Vec<String>,
    pub contents: Vec<Vec<String>>,
}


pub fn parse_data_from_file(filename: &str) -> Result<Data> {

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

    let profile_str = data.contents.get(0).context("No profile found")?.join("\n");
    let profile: Value = serde_json::from_str(&profile_str)?;

    for i in 1..data.cards.len() {

        let key = data.cards.get(i).context("No key found!")?;
        let value = data.contents.get(i).context("No value found!")?.join("\n");
        let json_value: Result<Value, _> = serde_json::from_str(&value);
        match json_value {
            Ok(value) => {
                let original_key = if let Some((trimmed, _)) = key.split_once('-') { trimmed } else { key };
                let item = json!({original_key: value});
                showcase.as_array_mut().unwrap().push(item);
            }
            Err(e) => return Err(anyhow!("error!")),
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


fn parse_data(json_data: &Value) -> Data {

    let profile = &json_data["profile"];
    let showcase = &json_data["showcase"];

    let arr = showcase.as_array().unwrap();

    // .map(|v| serde_json::to_string(v).unwrap())

    let mut cards = vec!["Profile".to_string()];
    let mut contents = vec![format_json_value(&profile)];

    // Iterate over each object in the array
    for obj in arr {
        if let Value::Object(map) = obj {
            // Iterate over key-value pairs in the object
            for (key, value) in map {
                // if let value_map = serde_json::from_str(value) {
                let key_str = if key != "Section"  {
                    if let Some(shape) = value.get("shape").and_then(Value::as_str) {
                        format!("{}-{}", key, shape)
                    } else {
                        format!("{}-2x2", key)
                    }
                } else {
                    format!("{}-1x8", key)
                };
                cards.push(key_str);
                contents.push(format_json_value(value));
                // }
            }
        }
    }

    Data { cards, contents }
}

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
