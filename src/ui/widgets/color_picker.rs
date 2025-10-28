use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::{Emoji, Theme};

pub struct ColorPicker {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub selected_channel: usize, // 0=R, 1=G, 2=B
}

impl ColorPicker {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            selected_channel: 0,
        }
    }

    pub fn adjust(&mut self, delta: i16) {
        let current = match self.selected_channel {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            _ => return,
        };

        *current = (*current as i16 + delta).clamp(0, 255) as u8;
    }

    pub fn next_channel(&mut self) {
        self.selected_channel = (self.selected_channel + 1) % 3;
    }

    pub fn prev_channel(&mut self) {
        self.selected_channel = if self.selected_channel == 0 {
            2
        } else {
            self.selected_channel - 1
        };
    }
}

pub fn render(picker: &ColorPicker, theme: &Theme, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} RGB Color Picker", Emoji::COLOR))
        .style(theme.title)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Color preview
    let preview = Paragraph::new(vec![
        Line::from(format!(
            "Current Color: {}",
            crate::ui::theme::color_indicator(picker.r, picker.g, picker.b)
        )),
        Line::from(format!(
            "Hex: #{:02X}{:02X}{:02X}",
            picker.r, picker.g, picker.b
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("Preview"));
    frame.render_widget(preview, chunks[1]);

    // RGB sliders
    let r_style = if picker.selected_channel == 0 {
        theme.highlight
    } else {
        theme.text
    };
    let g_style = if picker.selected_channel == 1 {
        theme.highlight
    } else {
        theme.text
    };
    let b_style = if picker.selected_channel == 2 {
        theme.highlight
    } else {
        theme.text
    };

    let sliders = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("🔴 Red:   ", r_style),
            Span::raw(format!("{:3} ", picker.r)),
            Span::raw("█".repeat((picker.r as usize * 20) / 255)),
        ]),
        Line::from(vec![
            Span::styled("🟢 Green: ", g_style),
            Span::raw(format!("{:3} ", picker.g)),
            Span::raw("█".repeat((picker.g as usize * 20) / 255)),
        ]),
        Line::from(vec![
            Span::styled("🔵 Blue:  ", b_style),
            Span::raw(format!("{:3} ", picker.b)),
            Span::raw("█".repeat((picker.b as usize * 20) / 255)),
        ]),
        Line::from(""),
        Line::from("Use Tab to switch channels"),
        Line::from("Use arrow keys to adjust"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Channels"));
    frame.render_widget(sliders, chunks[2]);

    // Help
    let help = Paragraph::new(
        "[Tab] Switch Channel  [↑↓] ±10  [Shift+↑↓] ±5  [Enter] Apply  [Esc] Cancel",
    )
    .style(theme.dim)
    .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(help, chunks[3]);
}
