use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub struct Theme {
    pub title: Style,
    pub border: Style,
    pub border_focused: Style, // Focused pane border
    pub highlight: Style,
    pub text: Style,
    pub success: Style,
    pub error: Style,
    pub dim: Style,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD), // Blue border for focused pane
            highlight: Style::default().fg(Color::Black).bg(Color::Cyan),
            text: Style::default().fg(Color::White),
            success: Style::default().fg(Color::Green),
            error: Style::default().fg(Color::Red),
            dim: Style::default().fg(Color::DarkGray),
        }
    }
}

pub struct Emoji;

impl Emoji {
    pub const POWER_ON: &'static str = "✅";
    pub const POWER_OFF: &'static str = "⭕";
    pub const LIGHT: &'static str = "💡";
    pub const COLOR: &'static str = "🌈";
    pub const WARM: &'static str = "☀️";
    pub const COOL: &'static str = "❄️";
    pub const BRIGHTNESS: &'static str = "🔆";
    pub const HELP: &'static str = "❓";
    pub const HOME: &'static str = "🏠";
    pub const DEVICE: &'static str = "📱";
    pub const LOADING: &'static str = "⏳";
}

pub fn brightness_bar(value: u8, width: usize) -> String {
    let filled = (value as usize * width) / 100;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// A block of terminal cells painted with the actual RGB color
pub fn color_swatch(r: u8, g: u8, b: u8, width: usize) -> Span<'static> {
    Span::styled(" ".repeat(width), Style::default().bg(Color::Rgb(r, g, b)))
}

pub fn color_indicator(r: u8, g: u8, b: u8) -> Line<'static> {
    Line::from(vec![
        color_swatch(r, g, b, 6),
        Span::raw(format!(" RGB({r},{g},{b}) #{r:02X}{g:02X}{b:02X}")),
    ])
}

pub fn temp_indicator(kelvin: u16) -> String {
    if kelvin < 3500 {
        format!("{}K {}", kelvin, Emoji::WARM)
    } else if kelvin > 6000 {
        format!("{}K {}", kelvin, Emoji::COOL)
    } else {
        format!("{kelvin}K")
    }
}
