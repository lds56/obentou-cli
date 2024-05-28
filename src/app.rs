use crate::arrange::{arrange_grid, CellSize};
use crate::card::Card;
use crate::config::Config;
use crate::data::{parse_data_from_file, save_data_to_file, Data};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::{canvas::*, *};
use tui_textarea::TextArea;

use std::fs::OpenOptions;
use std::io;
use std::io::Write;

use anyhow::{Context, Result};

use crate::write_info;

pub struct App {
    data: Data,
    config: Config,
    tui_state: TuiState,
    message: String,
    oops_count: usize,
    text_area: TextArea<'static>,
    source_file: String,
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
        let items = if !filename.is_empty() {
            parse_data_from_file(&filename)?
        } else {
            vec![]
        };

        let tui_state = TuiState::Select(0);
        let message = "ok".to_string();
        let oops_count = 0;
        let text_area = TextArea::new(items.get(0).context("Empty data")?.get_lines().to_vec());

        let config = Config::load("metadata.toml")?;
        let data = Data {
            metadata: config.get_metadata().clone(),
            items,
        };

        write_info!("Initialize app...");

        Ok(Self {
            data,
            config,
            tui_state,
            message,
            oops_count,
            text_area,
            source_file: filename,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        write_info!("Start to run loop..");

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
            let titles_list = self.data.items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    
                    let mut style = Style::default().fg(*self.data.metadata.get_card_color(&item.get_title()));

                    if let TuiState::Select(selected_index) = self.tui_state {
                        if i == selected_index {
                            style = Style::default().bg(Color::Yellow).fg(Color::Black);
                        }
                    }

                    let prefix = if i == 0 { ">".to_string() } else { format!("{}.", i) };
                    let title_and_shape = if i == 0 {
                        item.get_title().to_string()
                    } else {
                        format!("{}-{}", item.get_title(), item.get_shape())
                    };
                    ListItem::new(Span::styled(String::from(
                        format!("{} {}", prefix, title_and_shape)
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

            if let TuiState::Edit(selected_index) = self.tui_state {

                let title = self.data.items
                                .get(selected_index).expect("Item not found!")
                                .get_title();
                
                let json_str = self.text_area.lines().join("\n");
                let is_valid = self.data.metadata.is_valid(&json_str, title);

                match is_valid {
                    Ok(()) => {
                        self.oops_count = 0;
                        editor_title = String::from("OK");
                        editor_style = Style::default().fg(Color::Green);
                    }
                    Err(msg) => {
                        let ooo = "o".repeat(self.oops_count);
                        editor_title = format!("O{ooo}ps! {msg}");
                        editor_style = Style::default().fg(Color::Red);
                    }
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

                    let main_cards: Vec<String> = self.data.items
                                              .iter()
                                              .skip(1)
                                              .map(|item| format!("{}-{}", item.get_title(), item.get_shape()))
                                              .collect();

                    let cell_size_list: Vec<CellSize> = arrange_grid((50, 8), &main_cards);

                    // write_info!(format!("cell size list: {:?}", cell_size_list));

                    let (selected_y, selected_h) = if selected_index >= 1 && selected_index-1 < cell_size_list.len() {
                        (start_y + (cell_size_list[selected_index-1].get_start_row() * 20) as f64 + gap_y,
                         (cell_size_list[selected_index-1].get_height() * 20) as f64 - gap_y * 2.0)
                    } else {
                        (0f64, 0f64)
                    };

                    let offset_y = if (max_y as f64) < selected_y + selected_h {
                        selected_y + selected_h - max_y
                    } else {
                        0f64
                    };

                    for (i, cell_size) in cell_size_list.iter().enumerate() {
                        let x = start_x + (cell_size.get_start_col() * 20) as f64 + gap_x;
                        let y = start_y + (cell_size.get_start_row() * 20) as f64 + gap_y;
                        let w = (cell_size.get_width() * 20) as f64 - gap_x * 2.0;
                        let h = (cell_size.get_height() * 20) as f64 - gap_y * 2.0;

                        // if i+1 != selected_index {
                            ctx.draw(&Card {
                                x,
                                y: max_y - (y + h - offset_y),
                                width: w,
                                height: h,
                                color: if i+1 != selected_index {
                                    *self.data.metadata.get_card_color(cell_size.get_card_type())
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
                    self.data.metadata.get_cards()
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
                    self.data.metadata.get_shapes()
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
            TuiState::Create(selected_index, card_index, shape_index) => {
                self.create_mode(key_event, selected_index, card_index, shape_index)
            }
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
                let title = self
                    .data
                    .items
                    .get(selected_index)
                    .context("Index out of bound")?
                    .get_title();
                write_info!(format!("Edit {}...", title));

                let json_str = self.text_area.lines().join("\n");
                let is_valid = self.data.metadata.is_valid(&json_str, title);
                write_info!(format!("edit is valid? {:?}", is_valid));

                match is_valid {
                    Ok(()) => {
                        self.oops_count = 0;
                        self.tui_state = TuiState::Select(selected_index);
                        self.data
                            .items
                            .get_mut(selected_index)
                            .context("No item found!")?
                            .set_lines_and_format(self.text_area.lines());
                    }
                    Err(_) => {
                        // self.message = msg.to_string();
                        self.oops_count += 1;
                    }
                }
            }
            _ => {
                self.text_area.input(key_event);
            }
        }
        Ok(())
    }

    fn select_mode(&mut self, key_event: KeyEvent, selected_index: usize) -> Result<()> {
        write_info!(format!("> Select - index: {}", selected_index));
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
                if selected_index < self.data.items.len() - 1 {
                    self.data.items.swap(selected_index, selected_index + 1);
                    self.tui_state = TuiState::Select(selected_index + 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Char('K') => {
                if selected_index > 0 {
                    self.data.items.swap(selected_index, selected_index - 1);
                    self.tui_state = TuiState::Select(selected_index - 1);
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let item = self
                    .data
                    .items
                    .get_mut(selected_index)
                    .context("No item found!")?;

                if item.get_title() != "Section" {
                    let mut shape_index = self.data.metadata.index_of_shape(item.get_shape());
                    shape_index = (shape_index + 1) % self.data.metadata.count_shapes();

                    let new_shape = self
                        .data
                        .metadata
                        .get_shape(shape_index)
                        .context("No item shape found!")?;
                    item.set_shape(new_shape.to_string());
                    write_info!(format!(
                        "> Reshape - {}-{}",
                        item.get_title(),
                        item.get_shape()
                    ));
                }
            }
            KeyCode::Enter => {
                self.tui_state = TuiState::Edit(selected_index);
                self.oops_count = 0;
                self.text_area =
                    TextArea::new(self.data.items[selected_index].get_lines().to_vec());
            }
            KeyCode::Up => {
                if selected_index > 0 {
                    self.tui_state = TuiState::Select(selected_index - 1);
                    self.text_area =
                        TextArea::new(self.data.items[selected_index - 1].get_lines().to_vec());
                }
            }
            KeyCode::Down => {
                if selected_index < self.data.items.len() - 1 {
                    self.tui_state = TuiState::Select(selected_index + 1);
                    self.text_area =
                        TextArea::new(self.data.items[selected_index + 1].get_lines().to_vec());
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn create_mode(
        &mut self,
        key_event: KeyEvent,
        selected_index: usize,
        card_index: usize,
        shape_index: usize,
    ) -> Result<()> {
        write_info!(format!("create: {}", selected_index));
        match key_event.code {
            KeyCode::Enter => {
                if shape_index == 999 && card_index != 0 {
                    self.tui_state = TuiState::Create(selected_index, card_index, 0);
                } else {
                    // create new item
                    let new_item = self
                        .data
                        .metadata
                        .create_item(card_index, shape_index)
                        .context("Failed to create new item")?;

                    // insert to data
                    self.data.items.insert(selected_index + 1, new_item);
                    write_info!(format!("create - idx: {}", selected_index + 1));

                    self.tui_state = TuiState::Edit(selected_index + 1);
                    self.text_area =
                        TextArea::new(self.data.items[selected_index + 1].get_lines().to_vec());
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
                        self.tui_state =
                            TuiState::Create(selected_index, card_index, shape_index - 1);
                    }
                }
            }
            KeyCode::Down => {
                if shape_index == 999 {
                    if card_index < self.data.items.len() - 1 {
                        self.tui_state = TuiState::Create(selected_index, card_index + 1, 999);
                    }
                } else {
                    if shape_index < self.data.metadata.count_shapes() - 1 {
                        self.tui_state =
                            TuiState::Create(selected_index, card_index, shape_index + 1);
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
                    // remove deleted item
                    self.data.items.remove(selected_index);

                    self.tui_state = TuiState::Select(selected_index - 1);
                    self.text_area =
                        TextArea::new(self.data.items[selected_index - 1].get_lines().to_vec());
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

        write_info!("App dropped...");
        
        // Restore terminal
        disable_raw_mode().expect("Failed to disable raw mode");
        crossterm::execute!(io::stdout(), LeaveAlternateScreen)
            .expect("Failed to leave alternate screen");

        // Save data to file
        if let Err(e) = save_data_to_file(&self.data, &self.source_file) {
            eprintln!("Error saving data: {}", e);
        }
    }
}
