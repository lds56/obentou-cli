use ratatui::{
    style::Color,
    widgets::canvas::{Line, Painter, Shape},
};

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
