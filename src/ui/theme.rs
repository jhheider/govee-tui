use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub title: Style,
    pub border: Style,
    pub border_focused: Style,  // Focused pane border
    pub highlight: Style,
    pub text: Style,
    pub success: Style,
    #[allow(dead_code)]
    pub warning: Style,
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
                .add_modifier(Modifier::BOLD),  // Blue border for focused pane
            highlight: Style::default().fg(Color::Black).bg(Color::Cyan),
            text: Style::default().fg(Color::White),
            success: Style::default().fg(Color::Green),
            warning: Style::default().fg(Color::Yellow),
            error: Style::default().fg(Color::Red),
            dim: Style::default().fg(Color::DarkGray),
        }
    }
}

pub struct Emoji;

#[allow(dead_code)]
impl Emoji {
    pub const POWER_ON: &'static str = "✅";
    pub const POWER_OFF: &'static str = "⭕";
    pub const LIGHT: &'static str = "💡";
    pub const STRIP: &'static str = "🌡️";
    pub const BULB: &'static str = "💡";
    pub const COLOR: &'static str = "🌈";
    pub const WARM: &'static str = "☀️";
    pub const COOL: &'static str = "❄️";
    pub const BRIGHTNESS: &'static str = "🔆";
    pub const SETTINGS: &'static str = "⚙️";
    pub const HELP: &'static str = "❓";
    pub const REFRESH: &'static str = "🔄";
    pub const SEARCH: &'static str = "🔍";
    pub const HOME: &'static str = "🏠";
    pub const STATS: &'static str = "📊";
    pub const DEVICE: &'static str = "📱";
    pub const API: &'static str = "📡";
    pub const DATABASE: &'static str = "💾";
    pub const TIME: &'static str = "🕐";
    pub const SUCCESS: &'static str = "✓";
    pub const ERROR: &'static str = "✗";
    pub const ARROW_UP: &'static str = "↑";
    pub const ARROW_DOWN: &'static str = "↓";
    pub const ARROW_LEFT: &'static str = "←";
    pub const ARROW_RIGHT: &'static str = "→";
    pub const LOADING: &'static str = "⏳";
}

pub fn brightness_bar(value: u8, width: usize) -> String {
    let filled = (value as usize * width) / 100;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub fn color_indicator(r: u8, g: u8, b: u8) -> String {
    match (r, g, b) {
        (255, 0, 0) => "🔴 Red".to_string(),
        (0, 255, 0) => "🟢 Green".to_string(),
        (0, 0, 255) => "🔵 Blue".to_string(),
        (255, 255, 0) => "🟡 Yellow".to_string(),
        (255, 0, 255) => "🟣 Magenta".to_string(),
        (0, 255, 255) => "🔵 Cyan".to_string(),
        (255, 255, 255) => "⚪ White".to_string(),
        _ => format!("🌈 RGB({},{},{})", r, g, b),
    }
}

pub fn temp_indicator(kelvin: u16) -> String {
    if kelvin < 3500 {
        format!("{}K {}", kelvin, Emoji::WARM)
    } else if kelvin > 6000 {
        format!("{}K {}", kelvin, Emoji::COOL)
    } else {
        format!("{}K", kelvin)
    }
}
