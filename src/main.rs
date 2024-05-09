mod card;
mod arrange;

use crate::card::Card; // Import the Card struct
use crate::arrange::{CellSize, arrange_grid};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::{canvas::*, *};
use tui_textarea::TextArea;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::fmt;
use std::io;

use serde_json::{Value,  json};

static CARDS: &'static [&str] = &["Section", "Note", "Social", "Link", "Album", "Photo", "Counter", "Map"];
static SHAPES: &'static [&str] = &["4x4", "4x2", "2x4", "2x2", "1x4"];

static COLORS: &[Color] = &[
    Color::LightRed,
    Color::LightGreen,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
    Color::Indexed(213),
    Color::Indexed(202),
];

fn get_card_color(card: &str) -> Color {
    if let Some(index) = CARDS.iter().position(|x| *x == card) {
        COLORS[index]
    } else {
        Color::Gray
    }
}

enum TuiState {
    Select(usize),
    Edit(usize),
    Create(usize, usize, usize),
    Delete(usize),
}

#[derive(Debug, Default, Clone)]
struct Data {
    cards: Vec<String>,
    contents: Vec<Vec<String>>,
}


fn format_json(input: &str) -> Vec<String> {
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

macro_rules! write_info {
    ($content:expr) => {{
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("log.txt")
            .expect("Failed to open file");

        writeln!(file, "{}", $content).expect("Failed to write to file");
    }};
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


fn parse_data_from_file(filename: &str) -> Result<Data, Box<dyn std::error::Error>> {

    // write_info!(format!("read file: {}", filename));

    let mut file = File::open(filename)?;

    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Failed to read file");

    let data_json: Value = serde_json::from_str(&data)?;
    
    Ok(parse_data(&data_json))
}

fn save_data_to_file(data: &Data, filename: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut showcase = json!([]);

    let profile_str = data.contents.get(0).ok_or("No profile found")?.join("\n");
    let profile: Value = serde_json::from_str(&profile_str)?;

    for i in 1..data.cards.len() {

        let key = data.cards.get(i).ok_or("No key found!")?;
        let value = data.contents.get(i).ok_or("No value found!")?.join("\n");
        let json_value: Result<Value, _> = serde_json::from_str(&value);
        match json_value {
            Ok(value) => {
                let original_key = if let Some((trimmed, _)) = key.split_once('-') { trimmed } else { key };
                let item = json!({original_key: value});
                showcase.as_array_mut().unwrap().push(item);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    let output_data = json!({
        "profile": profile,
        "showcase": showcase,
    });

    write_info!(format!("output: {}", output_data));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)?;

    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer_pretty(&mut file, &output_data)?;

    Ok(())
}

fn create_card(card_index: usize) -> Option<Vec<String>> {
    if card_index >= CARDS.len() {
        return None;
    }
    Some(vec!["{}".to_string()])
}

fn main() -> Result<(), io::Error> {

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut data = if args.len() >= 2 {
        parse_data_from_file(filename).unwrap()
    } else {
        Data::default()
    };  // TODO: error handling

    // write_info!("Application started");
    write_info!(format!("data: {}", data));

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let titles = &mut data.cards;
    let contents = &mut data.contents;
    
    let mut tui_state = TuiState::Select(0);
    
    let mut oops_count = 0;
    let mut text_area = TextArea::new(contents[0].clone());


    // let card_options: Vec<&str> = vec!["Section", "Note", "Social", "Link", "Album", "Photo", "Map"];

    loop {
        terminal.draw(|f| {
            let size = f.size();
            // 创建三列布局
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20), // 第一列宽度
                    Constraint::Percentage(50), // 第二列宽度
                    Constraint::Percentage(30), // 第三列宽度
                ])
                .split(size);

            // 第一列：标题列表
            let titles_list = titles
                .iter()
                .enumerate()
                .map(|(i, title)| {
                    let original_title = if let Some((trimmed, _)) = title.split_once('-') { trimmed } else { title };
                    let mut style = Style::default().fg(get_card_color(original_title));

                    if let TuiState::Select(selected_index) = tui_state {
                        if i == selected_index {
                            style = Style::default().bg(Color::Yellow).fg(Color::Black);
                        }
                    }


                    let prefix = if i == 0 { ">".to_string() } else { format!("{}.", i) };
                    ListItem::new(Span::styled(String::from(
                        format!("{} {}", prefix, title)
                    ), style))
                })
                .collect::<Vec<_>>();

            let titles_widget = List::new(titles_list)
                .block(Block::default().title(Span::styled(
                    "Title",
                    if let TuiState::Select(_) = tui_state {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )).borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));

            f.render_widget(titles_widget, chunks[0]);

            // 第二列：内容编辑器
            let mut editor_title = String::from("Edit");
            let mut editor_style = Style::default();
            if let TuiState::Edit(_) = tui_state {
                let json_valid = text_area.lines().join("\n").is_valid_json();
                if json_valid {
                    oops_count = 0;
                    editor_title = String::from("OK");
                    editor_style = Style::default().fg(Color::Green);
                } else {
                    let ooo = "o".repeat(oops_count);
                    editor_title = format!("O{ooo}ps! Invalid JSON");
                    editor_style = Style::default().fg(Color::Red);
                }
            }

            text_area.set_block(
                Block::default()
                    .title(Span::styled(
                        editor_title,
                        editor_style,
                    ))
                    .borders(Borders::ALL),
            );

            text_area.set_line_number_style(Style::default().fg(Color::DarkGray));

            f.render_widget(text_area.widget(), chunks[1]);

            // 第三列：内容预览
            let max_x = 200.0;
            let max_y = 500.0;
            let preview = Canvas::default()
                .marker(symbols::Marker::HalfBlock)
                .block(Block::default().title("Preview").borders(Borders::ALL))
                .x_bounds([0.0, max_x])
                .y_bounds([0.0, max_y])
                .paint(|ctx| {

                    let selected_index = match tui_state {
                        TuiState::Edit(idx) => idx,
                        TuiState::Select(idx) => idx,
                        TuiState::Create(idx, _, _) => idx,
                        TuiState::Delete(idx) => idx,
                    };

                    let start_x = 20.0;
                    let start_y = 5.0;
                    let gap_x = 5.0;
                    let gap_y = 5.0;

                    write_info!(format!("cell list: {:?}", &titles[1..]));

                    let cell_size_list = arrange_grid((50, 8), &titles[1..]);

                    write_info!(format!("cell size list: {:?}", cell_size_list));

                    for (i, cell_size) in cell_size_list.iter().enumerate() {
                        let x = start_x + (cell_size.get_start_col() * 20) as f64 + gap_x;
                        let y = start_y + (cell_size.get_start_row() * 20) as f64 + gap_y;
                        let w = (cell_size.get_width() * 20) as f64 - gap_x * 2.0;
                        let h = (cell_size.get_height() * 20) as f64 - gap_y * 2.0;

                        // if i+1 != selected_index {
                            ctx.draw(&Card {
                                x,
                                y: max_y - y - h,
                                width: w,
                                height: h,
                                color: if i+1 != selected_index {
                                    get_card_color(cell_size.get_card_type())
                                } else {
                                    Color::Yellow
                                },
                            });
                        /*} else {
                            ctx.draw(&Rectangle {
                                x: x,
                                y: max_y - y - h,
                                width: w,
                                height: h,
                                color: get_card_color(cell_size.get_card_type()),
                            });

                        }*/
                    }

                     // Draw rectangles

                    
                    // ctx.print(start_x + 20.0, max_y - start_y + 20.0 , "1".white()); // Print text on rectangle
                });
            
            f.render_widget(preview, chunks[2]);
            
            // 底部状态栏
            let status_bar_text = match tui_state {
                TuiState::Edit(_) => "Shortcuts: Go Back(Esc)",
                TuiState::Select(_) => "Shortcuts: Move Cursor(↑↓) Select(↵) Create New(N) Delete(D) Quit(Q)",
                TuiState::Create(_, _, _) => "Shortcuts: Move Cursor(↑↓) Confirm Create(↵) Cancel(Esc)",
                TuiState::Delete(_) => "Shortcuts: Confirm Delete(↵) Cancel(Esc)"
            };
            
            let status_bar = Paragraph::new(Span::raw(status_bar_text))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status_bar, Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
            .split(size)[1]);

            if let TuiState::Create(_, card_index, shape_index) = tui_state {

                let center_area = centered_rect(20, 25, size);

                let block = Block::default()
                    .title("Create")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan));

                let items: Vec<ListItem> = if shape_index == 999 {
                    CARDS
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let style = if i == card_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(Span::styled(*option, style))
                    })
                    .collect()
                } else {
                    SHAPES
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let style = if i == shape_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(Span::styled(*option, style))
                    })
                    .collect()
                };

                let list = List::new(items).block(block);
                f.render_widget(Clear, center_area);
                f.render_widget(list, center_area);
            }
        
            if let TuiState::Delete(_) = tui_state {

                let center_area = centered_rect(20, 10, size);

                let block = Block::default()
                    .title("Delete")
                    .title_bottom(text::Line::from("Confirm").left_aligned())
                    .title_bottom(text::Line::from("Cancel").right_aligned())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red));

            // let list = List::new(items).block(block);
                f.render_widget(Clear, center_area);
                f.render_widget(block, center_area);
            }

        })?;

        // 处理事件
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') |
                KeyCode::Char('Q') => {
                    if let TuiState::Edit(_) = tui_state {
                        text_area.input(key_event);
                    } else {
                        break;
                    }
                },
                KeyCode::Char('n') |
                KeyCode::Char('N') => {
                    match tui_state {
                        TuiState::Edit(_) => { text_area.input(key_event); },
                        TuiState::Select(selected_index) => {
                            tui_state = TuiState::Create(selected_index, 0, 999);
                        },
                        _ => (),
                    }
                },
                KeyCode::Char('d') |
                KeyCode::Char('D') => {
                    match tui_state {
                        TuiState::Edit(_) => { text_area.input(key_event); },
                        TuiState::Select(selected_index) => {
                            if selected_index != 0 {
                                tui_state = TuiState::Delete(selected_index);
                            }
                        },
                        _ => (),
                    }
                },

                // KeyCode::Char('s') |
                // KeyCode::Char('S') => {
                   //  if editing_mode {
                      //  text_area.input(key_event);
                    // } else {
                       //  write_info!(format!("Save data: {}", data));
                    // }
                // },
                KeyCode::Enter => {
                    match tui_state {
                        TuiState::Edit(_) => { text_area.input(key_event); },
                        TuiState::Select(selected_index) => {
                            tui_state = TuiState::Edit(selected_index);
                            oops_count = 0;
                            // contents[selected_index] = text_area.lines().to_vec();
                        },
                        TuiState::Create(selected_index, card_index, shape_index) => {
                            if shape_index == 999 {
                                tui_state = TuiState::Create(selected_index, card_index, 0);
                            } else {
                                if let Some(card) = create_card(card_index) {
                                    
                                    contents.insert(selected_index + 1, card);

                                    let title_str = format!("{}-{}", CARDS[card_index], SHAPES[shape_index]);
                                    titles.insert(selected_index + 1, title_str);

                                    tui_state = TuiState::Edit(selected_index + 1);
                                    text_area = TextArea::new(contents[selected_index + 1].clone());
                                }
                            }
                        },
                        TuiState::Delete(selected_index) => {
                            contents.remove(selected_index);
                            titles.remove(selected_index);
                            tui_state = TuiState::Select(selected_index-1);
                        },
                    }
                }
                KeyCode::Esc => {
                    match tui_state {
                        TuiState::Edit(selected_index) => {
                            let json_valid = text_area.lines().join("\n").is_valid_json();
                            if json_valid {
                                tui_state = TuiState::Select(selected_index);
                                contents[selected_index] = format_json(text_area.lines().join("\n").as_str());
                            } else {
                                oops_count += 1;
                            }
                        }
                        TuiState::Create(selected_index, _, _) => {
                            tui_state = TuiState::Select(selected_index);
                        }
                        TuiState::Select(_) => (),
                        TuiState::Delete(selected_index) => {
                            tui_state = TuiState::Select(selected_index);
                        },
                    }
                }
                KeyCode::Up => {
                    match tui_state {
                        TuiState::Edit(_) => { text_area.input(key_event); },
                        TuiState::Select(selected_index) => {
                            if selected_index > 0 {
                                tui_state = TuiState::Select(selected_index - 1);
                                text_area = TextArea::new(contents[selected_index - 1].clone());
                            }
                        },
                        TuiState::Create(selected_index, card_index, shape_index) => {
                            if shape_index == 999 {
                                if card_index > 0 {
                                    tui_state = TuiState::Create(selected_index, card_index - 1, 999);
                                }
                            } else {
                                if shape_index > 0 {
                                    tui_state = TuiState::Create(selected_index, card_index, shape_index - 1);
                                }
                            }
                        }
                        TuiState::Delete(_) => (),
                    }
                }
                KeyCode::Down => {
                    match tui_state {
                        TuiState::Edit(_) => { text_area.input(key_event); },
                        TuiState::Select(selected_index) => {
                            if selected_index < titles.len() - 1 {
                                tui_state = TuiState::Select(selected_index + 1);
                                text_area = TextArea::new(contents[selected_index + 1].clone());
                            }
                        },
                        TuiState::Create(selected_index, card_index, shape_index) => {
                            if shape_index == 999 {
                                if card_index < CARDS.len() - 1 {
                                    tui_state = TuiState::Create(selected_index, card_index + 1, 999);
                                }
                            } else {
                                if shape_index < SHAPES.len() - 1 {
                                    tui_state = TuiState::Create(selected_index, card_index, shape_index + 1);
                                }
                            }
                        },
                        TuiState::Delete(_) => (),
                    }
                }
                _ => {
                    if let TuiState::Edit(_) = tui_state {
                        text_area.input(key_event);
                    }
                }
            }
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    write_info!(format!("new data: {}", data));

    let res = save_data_to_file(&data, filename);  // TODO: error handling
    if res.is_err() {
        write_info!(format!("err: {}", res.unwrap_err()));
    }

    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

trait JsonValidation {
    fn is_valid_json(&self) -> bool;
}

impl JsonValidation for String {
    fn is_valid_json(&self) -> bool {
        serde_json::from_str::<serde_json::Value>(self).is_ok()
    }
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



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_arrange() {

        write_info!("test arrange");
        
        // let cards = vec!["Link-2x2", "Map-2x4", "Counter-1x4", "Link-2x4",
           //              "Section-1x8", "Note-2x2", "Album-4x4"];

        let string_array = [
            "Section-1x8", "Note-4x4", "Note-4x2", "Note-2x4", "Social-2x2", "Counter-1x4", "Section-1x8",
            "Social-2x2", "Social-2x4", "Link-1x4", "Link-2x4", "Album-4x4", "Section-1x8", "Photo-4x2",
            "Section-1x8",
        ];

        let cards: Vec<String> = string_array.iter().map(|s| String::from(*s)).collect();


        let l = arrange_grid((50, 8), &cards);
        write_info!(format!("len: {}", l.len()));
        for c in l.iter() {
            write_info!(format!("{:?}", c));
        }


    // assert_eq!(adder(-2, 3), 1);
    }
}
