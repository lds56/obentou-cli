use crate::data::{
    Data, save_data_to_file, parse_data_from_file,
    format_json
};
use crate::card::Card;
use crate::arrange::{CellSize, arrange_grid};
use crate::config::Config;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::{canvas::*, *};
use tui_textarea::TextArea;

use std::io;
use std::io::Write;
use std::fs::OpenOptions;

use anyhow::{Context, Result};

use crate::write_info;

pub struct App {
    data: Data,
    config: Config,
    tui_state: TuiState,
    oops_count: usize,
    text_area: TextArea<'static>,
}

enum TuiState {
    Select(usize),
    Edit(usize),
    Create(usize, usize, usize),
    Delete(usize),
    Quit,
}

impl App {
    pub fn new(filename: String) -> Result<Self> {
        let data = if !filename.is_empty() {
            parse_data_from_file(&filename)?
        } else {
            Data::default()
        };

        let tui_state = TuiState::Select(0);
        let oops_count = 0;
        let text_area = TextArea::new(data.contents[0].clone());

        let config = Config::new("metadata.toml")?;

        Ok(Self {
            data,
            config,
            tui_state,
            oops_count,
            text_area,
        })
    }

    pub fn run(&mut self) -> Result<()> {

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            if let TuiState::Quit = self.tui_state {
                return Ok(());
            }
            self.render(&mut terminal)?;
            self.handle_input()?;
        }
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {

        // let titles = &mut self.data.cards;
        // let contents = &mut self.data.contents;

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
            let titles_list = self.data.cards
                .iter()
                .enumerate()
                .map(|(i, title)| {
                    let original_title = if let Some((trimmed, _)) = title.split_once('-') { trimmed } else { title };
                    let mut style = Style::default().fg(self.config.get_card_color(original_title));

                    if let TuiState::Select(selected_index) = self.tui_state {
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
                    if let TuiState::Select(_) = self.tui_state {
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
            if let TuiState::Edit(_) = self.tui_state {
                let json_valid = self.text_area.lines().join("\n").is_valid_json();
                if json_valid {
                    self.oops_count = 0;
                    editor_title = String::from("OK");
                    editor_style = Style::default().fg(Color::Green);
                } else {
                    let ooo = "o".repeat(self.oops_count);
                    editor_title = format!("O{ooo}ps! Invalid JSON");
                    editor_style = Style::default().fg(Color::Red);
                }
            }

            self.text_area.set_block(
                Block::default()
                    .title(Span::styled(
                        editor_title,
                        editor_style,
                    ))
                    .borders(Borders::ALL),
            );

            self.text_area.set_line_number_style(Style::default().fg(Color::DarkGray));

            f.render_widget(self.text_area.widget(), chunks[1]);

            // 第三列：内容预览
            let max_x = 200.0;
            let max_y = 500.0;
            let preview = Canvas::default()
                .marker(symbols::Marker::HalfBlock)
                .block(Block::default().title("Preview").borders(Borders::ALL))
                .x_bounds([0.0, max_x])
                .y_bounds([0.0, max_y])
                .paint(|ctx| {

                    let selected_index = match self.tui_state {
                        TuiState::Edit(idx) => idx,
                        TuiState::Select(idx) => idx,
                        TuiState::Create(idx, _, _) => idx,
                        TuiState::Delete(idx) => idx,
                        _ => 0,
                    };

                    let start_x = 20.0;
                    let start_y = 5.0;
                    let gap_x = 5.0;
                    let gap_y = 5.0;

                    // write_info!(format!("cell list: {:?}", &self.data.cards[1..]));

                    let cell_size_list: Vec<CellSize> = arrange_grid((50, 8), &self.data.cards[1..]);

                    // write_info!(format!("cell size list: {:?}", cell_size_list));

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
                                    self.config.get_card_color(cell_size.get_card_type())
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
                });

            f.render_widget(preview, chunks[2]);

            // 底部状态栏
            let status_bar_text = match self.tui_state {
                TuiState::Edit(_) => "Shortcuts: Go Back(Esc)",
                TuiState::Select(_) => "Shortcuts: Move Cursor(↑↓) Select(↵) Move Card(JK) Reshape Card(R) Create New(N) Delete(D) Quit(Q)",
                TuiState::Create(_, _, _) => "Shortcuts: Move Cursor(↑↓) Confirm Create(↵) Cancel(Esc)",
                TuiState::Delete(_) => "Shortcuts: Confirm Delete(↵) Cancel(Esc)",
                TuiState::Quit => "Bye~"
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

            if let TuiState::Create(_, card_index, shape_index) = self.tui_state {

                let center_area = centered_rect(20, 25, size);

                let block = Block::default()
                    .title("Create")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan));

                let items: Vec<ListItem> = if shape_index == 999 {
                    self.config.get_cards()
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let style = if i == card_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(Span::styled(option, style))
                    })
                    .collect()
                } else {
                    self.config.get_shapes()
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let style = if i == shape_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(Span::styled(option, style))
                    })
                    .collect()
                };

                let list = List::new(items).block(block);
                f.render_widget(Clear, center_area);
                f.render_widget(list, center_area);
            }

            if let TuiState::Delete(_) = self.tui_state {

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
        // ...
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        if let Event::Key(key_event) = event::read()? {
            self.process_key_event(key_event)?;
        }
        Ok(())
    }

    fn process_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.tui_state {
            TuiState::Edit(selected_index) => self.edit_mode(key_event, selected_index),
            TuiState::Select(selected_index) => self.select_mode(key_event, selected_index),
            TuiState::Create(selected_index, card_index, shape_index) => self.create_mode(
                key_event, selected_index, card_index, shape_index),
            TuiState::Delete(selected_index) => self.delete_mode(key_event, selected_index),
            _ => Ok(()),
        }
    }

    /*
    fn edit_mode(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key_event.code {
            KeyCode::Esc => {
                TuiState::Edit(selected_index) => {
                    let json_valid = text_area.lines().join("\n").is_valid_json();
                    if json_valid {
                        tui_state = TuiState::Select(selected_index);
                        contents[selected_index] = format_json(text_area.lines().join("\n").as_str());
                    } else {
                        oops_count += 1;
                    }
                }
            }
            _ => self.text_area.input(key_event),
        }
        Ok(())
    }

    fn select_mode(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key_event.code {
            // ... handle select mode key events
            _ => Ok(()),
        }
    }

    fn create_mode(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key_event.code {
            // ... handle create mode key events
            _ => Ok(()),
        }
    }

    fn delete_mode(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key_event.code {
            // ... handle delete mode key events
            _ => Ok(()),
        }
    }*/

    fn edit_mode(&mut self, key_event: KeyEvent, selected_index: usize) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                let json_valid = self.text_area.lines().join("\n").is_valid_json();
                if json_valid {
                    self.tui_state = TuiState::Select(selected_index);
                    self.data.contents[selected_index] =
                        format_json(self.text_area.lines().join("\n").as_str());
                } else {
                    self.oops_count += 1;
                }
            }
            _ => {
                self.text_area.input(key_event);
            }
        }
        Ok(())
    }

    fn select_mode(&mut self, key_event: KeyEvent, selected_index: usize) -> Result<()> {
        write_info!(format!("select: {}", selected_index));
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.tui_state = TuiState::Quit;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.tui_state = TuiState::Create(selected_index, 0, 999);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if selected_index != 0 {
                    self.tui_state = TuiState::Delete(selected_index);
                }
            }
            KeyCode::Char('j') | KeyCode::Char('J') => {
                if selected_index < self.data.cards.len() - 1 {
                    self.data.cards.swap(selected_index, selected_index+1);
                    self.data.contents.swap(selected_index, selected_index+1);
                    self.tui_state = TuiState::Select(selected_index+1);
                }
            }
            KeyCode::Char('k') | KeyCode::Char('K') => {
                if selected_index > 0 {
                    self.data.cards.swap(selected_index, selected_index-1);
                    self.data.contents.swap(selected_index, selected_index-1);
                    self.tui_state = TuiState::Select(selected_index-1);
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let title = &self.data.cards[selected_index];
                let (card, shape) = title.split_once('-').context("Invalid title")?;
                if card != "Section" {
                    let mut shape_index = self.config.get_shape_index(shape);
                    shape_index = (shape_index + 1) % self.config.get_shapes_size();
                    let new_shape = self.config.get_shape(shape_index);
                    self.data.cards[selected_index] = format!("{}-{}", card, new_shape);
                }
            }
            KeyCode::Enter => {
                self.tui_state = TuiState::Edit(selected_index);
                self.oops_count = 0;
                self.text_area = TextArea::new(self.data.contents[selected_index].clone());
            }
            KeyCode::Up => {
                if selected_index > 0 {
                    self.tui_state = TuiState::Select(selected_index - 1);
                    self.text_area = TextArea::new(self.data.contents[selected_index - 1].clone());
                }
            }
            KeyCode::Down => {
                if selected_index < self.data.cards.len() - 1 {
                    self.tui_state = TuiState::Select(selected_index + 1);
                    self.text_area = TextArea::new(self.data.contents[selected_index + 1].clone());
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn create_mode(&mut self, key_event: KeyEvent, selected_index: usize, card_index: usize, shape_index: usize) -> Result<()> {
        write_info!(format!("create: {}", selected_index));
        match key_event.code {
            KeyCode::Enter => {
                if shape_index == 999 && card_index != 0 {
                    self.tui_state = TuiState::Create(selected_index, card_index, 0);
                } else {

                    // insert new card to contents
                    self.data.contents.insert(selected_index + 1, self.config.create_card(card_index));
                    write_info!(format!("create - idx: {}", selected_index+1));

                    // insert new title to titles
                    let card_shape = if card_index == 0 { "1x8" } else { self.config.get_shape(shape_index) };
                    let title_str = format!("{}-{}", self.config.get_card(card_index), card_shape);
                    self.data.cards.insert(selected_index + 1, title_str);

                    self.tui_state = TuiState::Edit(selected_index + 1);
                    self.text_area = TextArea::new(self.data.contents[selected_index + 1].clone());
                }

            }
            KeyCode::Esc => {
                self.tui_state = TuiState::Select(selected_index);
            }
            KeyCode::Up => {
                if shape_index == 999 {
                    if card_index > 0 {
                        self.tui_state = TuiState::Create(selected_index, card_index - 1, 999);
                    }
                } else {
                    if shape_index > 0 {
                        self.tui_state = TuiState::Create(selected_index, card_index, shape_index - 1);
                    }
                }
            }
            KeyCode::Down => {
                if shape_index == 999 {
                    if card_index < self.config.get_cards_size() - 1 {
                        self.tui_state = TuiState::Create(selected_index, card_index + 1, 999);
                    }
                } else {
                    if shape_index < self.config.get_shapes_size() - 1 {
                        self.tui_state = TuiState::Create(selected_index, card_index, shape_index + 1);
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn delete_mode(&mut self, key_event: KeyEvent, selected_index: usize) -> Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                if selected_index != 0 {
                    self.data.contents.remove(selected_index);
                    self.data.cards.remove(selected_index);
                    self.tui_state = TuiState::Select(selected_index - 1);
                    self.text_area = TextArea::new(self.data.contents[selected_index - 1].clone());
                }
            }
            KeyCode::Esc => {
                self.tui_state = TuiState::Select(selected_index);
            }
            _ => (),
        }
        Ok(())
    }
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


impl Drop for App {
    fn drop(&mut self) {
        // Restore terminal
        disable_raw_mode().expect("Failed to disable raw mode");
        crossterm::execute!(io::stdout(), LeaveAlternateScreen).expect("Failed to leave alternate screen");

        // Save data to file
        if let Err(e) = save_data_to_file(&self.data, "output.json") {
            eprintln!("Error saving data: {}", e);
        }
    }
}


trait JsonValidation {
    fn is_valid_json(&self) -> bool;
}

impl JsonValidation for String {
    fn is_valid_json(&self) -> bool {
        serde_json::from_str::<serde_json::Value>(self).is_ok()
    }
}
