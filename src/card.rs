use ratatui::{
    style::Color,
    widgets::canvas::{Painter, Shape, Line},
};

pub mod constants {

    use ratatui::style::Color;

    pub static CARDS: &'static [&str] = &["Section", "Note", "Social", "Link", "Photo", "Album", "Counter", "Map"];
    pub static SHAPES: &'static [&str] = &["4x4", "4x2", "2x4", "2x2", "1x4"];

    // Grayscale
    pub static _COLORS: &[Color] = &[
        Color::Indexed(255),  // red
        Color::Indexed(252),  // pink
        Color::Indexed(249),  // orange
        Color::Indexed(246),   // blue
        Color::Indexed(243),   // cyan
        Color::Indexed(240),   // green
        Color::Indexed(237),   // dark green
        Color::Indexed(231),  // light yellow
    ];

    // Mondrian
    pub static MOND_COLORS: &[Color] = &[
        Color::Indexed(220),  // yellow
        Color::Indexed(027),  // blue
        Color::Indexed(016),  // gray
        Color::Indexed(124),  // red
        Color::Indexed(255),  // white
        Color::Indexed(220),  // yellow
        Color::Indexed(027),  // blue
        Color::Indexed(124),  // red
    ];

    // Soft
    pub static _SOFT_COLORS: &[Color] = &[
        Color::Indexed(175),
        Color::Indexed(104),
        Color::Indexed(116),
        Color::Indexed(115),
        Color::Indexed(150),
        Color::Indexed(186),
        Color::Indexed(180),
        Color::Indexed(174),
    ];
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Card {
    /// The `x` position of the card.
    ///
    /// The card is positioned from its bottom left corner.
    pub x: f64,
    /// The `y` position of the card.
    ///
    /// The card is positioned from its bottom left corner.
    pub y: f64,
    /// The width of the card.
    pub width: f64,
    /// The height of the card.
    pub height: f64,
    /// The color of the card.
    pub color: Color,
}
/*
impl Card {

    fn get_all_shapes() -> 'static &[&str] {
        SHAPES
    }
}
 */

impl Shape for Card {
    fn draw(&self, painter: &mut Painter) {

        let mut lines: Vec<Line> = vec![];
        for i in 0..(self.width as usize) {
            lines.push(Line {
                x1: self.x + i as f64,
                y1: self.y,
                x2: self.x + i as f64,
                y2: self.y + self.height,
                color: self.color,
            })
        }
        for line in &lines {
            line.draw(painter);
        }

    }
}


pub fn get_card_color(card: &str) -> Color {
    if let Some(index) = constants::CARDS.iter().position(|x| *x == card) {
        constants::MOND_COLORS[index]
    } else {
        Color::Gray
    }
}


pub fn create_card(card_index: usize) -> Option<Vec<String>> {
    if card_index >= constants::CARDS.len() {
        return None;
    }
    Some(vec!["{}".to_string()])
}

