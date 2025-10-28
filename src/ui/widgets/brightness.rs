use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::{self, Emoji, Theme};

pub struct BrightnessControl {
    pub value: u8,
}

impl BrightnessControl {
    pub fn new(value: u8) -> Self {
        Self {
            value: value.min(100),
        }
    }

    pub fn adjust(&mut self, delta: i16) {
        self.value = (self.value as i16 + delta).clamp(0, 100) as u8;
    }

    pub fn set(&mut self, value: u8) {
        self.value = value.min(100);
    }
}

pub fn render(brightness: &BrightnessControl, theme: &Theme, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} Brightness Control", Emoji::BRIGHTNESS))
        .style(theme.title)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Brightness bar
    let bar = theme::brightness_bar(brightness.value, 40);
    let control = Paragraph::new(vec![
        Line::from(format!("Level: {}%", brightness.value)),
        Line::from(""),
        Line::from(bar),
        Line::from(""),
        Line::from("[↑↓] Adjust ±5% [Shift+↑↓] Adjust ±1% [1-9] Set 10-90%"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Control"));
    frame.render_widget(control, chunks[1]);

    // Help
    let help = Paragraph::new("[Enter] Apply [Esc] Cancel")
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}
