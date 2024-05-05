use log::{info, error};
use simple_logger::SimpleLogger;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::{canvas::*, *};
use std::io;
use tui_textarea::TextArea;

use serde_json::Value;

fn format_json(input: &str) -> String {
    match serde_json::from_str::<Value>(input) {
        Ok(v) => {
            // 尝试重新格式化为漂亮的 JSON 字符串
            match serde_json::to_string_pretty(&v) {
                Ok(formatted_json) => formatted_json,
                Err(_) => input.to_string(), // 格式化失败，返回原始 JSON 字符串
            }
        }
        Err(_) => input.to_string(), // 解析失败，返回原始 JSON 字符串
    }
}

fn main() -> Result<(), io::Error> {

    SimpleLogger::new()
        .init()
        .unwrap();

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 测试数据
    let titles = vec![
        "今天天气真好",
        "明天是阴天",
        "后天大暴雨",
    ];
    let mut contents = vec![
        format_json(r#"{"today": "good day"}"#),
        format_json(r#"{"tomorrow": "bad day"}"#),
        format_json(r#"{"the day after tomorrow": "new day"}"#),
    ];

    for content in contents.iter() {
         info!("content: {}", content);
    }

    let mut selected_index = 0;
    let mut editing_mode = false;
    let mut oops_count = 0;
    let mut text_area = TextArea::new(vec![contents[selected_index].clone()]);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            // 创建三列布局
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20), // 第一列宽度
                    Constraint::Percentage(40), // 第二列宽度
                    Constraint::Percentage(40), // 第三列宽度
                ])
                .split(size);

            // 第一列：标题列表
            let titles_list = titles
                .iter()
                .enumerate()
                .map(|(i, title)| {
                    let style = if i == selected_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Span::styled(String::from(*title), style))
                })
                .collect::<Vec<_>>();

            let titles_widget = List::new(titles_list)
                .block(Block::default().title(Span::styled(
                    "标题",
                    if editing_mode {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                )).borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));

            f.render_widget(titles_widget, chunks[0]);

            // 第二列：内容编辑器
            let mut editor_title = String::from("编辑");
            let mut editor_style = Style::default();
            if editing_mode {
                let json_valid = text_area.lines().join("\n").is_valid_json();
                if json_valid {
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

            f.render_widget(text_area.widget(), chunks[1]);

            // 第三列：内容预览
            let preview = Canvas::default()
                .block(Block::default().title("Preview").borders(Borders::ALL))
                .paint(|ctx| {
                    let square_size = 2.0;
                    for row in 0..5 {
                        for col in 0..5 {
                            let x = col as f64 * square_size;
                            let y = row as f64 * square_size;
                            ctx.draw(&Rectangle {
                                x: 100.0+x,
                                y: 100.0+y,
                                width: square_size,
                                height: square_size,
                                color: Color::Red,
                            });
                        }
                    }
                });
            
            f.render_widget(preview, chunks[2]);
            
            // 底部状态栏
            let status_bar_text = if editing_mode {
                "快捷键: 返回(Esc)"
            } else {
                "快捷键: 移动(↑↓ ) 进入(↵ ) 退出(Q)"
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
        })?;

        // 处理事件
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => {
                    if editing_mode {
                        text_area.input(key_event);
                    } else {
                        break;
                    }
                },
                KeyCode::Enter => {
                    if editing_mode {
                        text_area.input(key_event);
                    }
                    else {
                        editing_mode = true;
                        oops_count = 0;
                        contents[selected_index] = text_area.lines().join("\n");
                        info!("contents: {}", contents[selected_index]);
                    }
                }
                KeyCode::Esc => {
                    if editing_mode {
                        let json_valid = text_area.lines().join("\n").is_valid_json();
                        if json_valid {
                            editing_mode = false;
                            contents[selected_index] = format_json(text_area.lines().join("\n").as_str());
                        } else {
                            oops_count += 1;
                        }
                    }
                }
                KeyCode::Up => {
                    if !editing_mode {
                        if selected_index > 0 {
                            selected_index -= 1;
                            text_area = TextArea::new(vec![contents[selected_index].clone()]);
                        }
                    } else {
                        text_area.input(key_event);
                    }
                }
                KeyCode::Down => {
                    if !editing_mode {
                        if selected_index < titles.len() - 1 {
                            selected_index += 1;
                            text_area = TextArea::new(vec![contents[selected_index].clone()]);
                        }
                    }
                }
                _ => {
                    if editing_mode {
                        text_area.input(key_event);
                    }
                }
            }
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

trait JsonValidation {
    fn is_valid_json(&self) -> bool;
}

impl JsonValidation for String {
    fn is_valid_json(&self) -> bool {
        serde_json::from_str::<serde_json::Value>(self).is_ok()
    }
}
