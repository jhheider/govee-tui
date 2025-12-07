use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{self, Emoji, Theme};

/// Create a temperature bar showing position between warm and cool
fn temp_bar(kelvin: u16, width: usize) -> String {
    // Map 2000-9000K to 0-width
    let position = ((kelvin.saturating_sub(2000)) as usize * width) / 7000;
    let position = position.min(width);
    format!(
        "{}{}{}",
        "░".repeat(position),
        "█",
        "░".repeat(width.saturating_sub(position + 1))
    )
}

pub fn render_with_style(
    device: &Device,
    state: Option<&DeviceState>,
    loading: bool,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    border_style: Style,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(4), // Info
            Constraint::Length(4), // Brightness
            Constraint::Length(5), // Color/Temp
            Constraint::Min(0),    // Controls
        ])
        .split(area);

    // Device header with emoji and loading indicator
    let device_emoji = if device.is_group {
        "📦"
    } else {
        Emoji::LIGHT
    };
    let loading_indicator = if loading { " ⏳" } else { "" };
    let header = Paragraph::new(format!(
        "{} {} ({}){}",
        device_emoji, device.name, device.model, loading_indicator
    ))
    .style(theme.title)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Power and basic info
    let power_status = if let Some(s) = state {
        if s.power {
            format!("{} ON", Emoji::POWER_ON)
        } else {
            format!("{} OFF", Emoji::POWER_OFF)
        }
    } else {
        "⏳ Loading...".to_string()
    };

    // Build capabilities list
    let mut caps = vec![];
    if device.supports_power {
        caps.push("⚡");
    }
    if device.supports_brightness {
        caps.push("☀️");
    }
    if device.supports_color {
        caps.push("🎨");
    }
    if device.supports_color_temp {
        caps.push("🌡️");
    }
    let caps_str = caps.join(" ");

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Power: "),
            Span::styled(&power_status, theme.text),
            Span::raw("   Caps: "),
            Span::styled(&caps_str, theme.dim),
        ]),
        Line::from(vec![
            Span::raw("Model: "),
            Span::styled(&device.model, theme.dim),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Info"));
    frame.render_widget(info, chunks[1]);

    // Brightness with visual bar
    if device.supports_brightness {
        let (brightness_text, bar_text) = if let Some(s) = state {
            if let Some(brightness) = s.brightness {
                let bar = theme::brightness_bar(brightness, 25);
                (format!("{}%", brightness), bar)
            } else {
                ("--".to_string(), String::new())
            }
        } else {
            ("...".to_string(), String::new())
        };

        let brightness_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(format!("{} ", Emoji::BRIGHTNESS), theme.text),
                Span::styled(&brightness_text, theme.title),
                Span::raw("  "),
                Span::styled("[↑↓ ±10%] [1-9 quick]", theme.dim),
            ]),
            Line::from(Span::raw(&bar_text)),
        ])
        .block(Block::default().borders(Borders::ALL).title("Brightness"));
        frame.render_widget(brightness_widget, chunks[2]);
    } else {
        let brightness_widget =
            Paragraph::new(Line::from(Span::styled("Not supported", theme.dim)))
                .block(Block::default().borders(Borders::ALL).title("Brightness"));
        frame.render_widget(brightness_widget, chunks[2]);
    }

    // Color and Temperature
    if device.supports_color || device.supports_color_temp {
        let mut lines = vec![];

        if let Some(s) = state {
            // Color line
            if device.supports_color {
                if let Some(color) = s.color {
                    lines.push(Line::from(vec![
                        Span::raw("🎨 "),
                        Span::styled(
                            theme::color_indicator(color.r, color.g, color.b),
                            theme.text,
                        ),
                        Span::styled("  [C] picker", theme.dim),
                    ]));
                } else {
                    lines.push(Line::from(Span::raw("🎨 Loading...")));
                }
            }

            // Temperature line with visual bar
            if device.supports_color_temp {
                if let Some(temp) = s.color_temp {
                    let temp_emoji = if temp < 3500 {
                        Emoji::WARM
                    } else if temp > 6000 {
                        Emoji::COOL
                    } else {
                        ""
                    };
                    lines.push(Line::from(format!(
                        "🌡️ {}K {}  [t/T] ±500K",
                        temp, temp_emoji
                    )));
                    lines.push(Line::from(format!("   WARM {} COOL", temp_bar(temp, 20))));
                } else {
                    lines.push(Line::from(Span::raw("🌡️ Loading...")));
                }
            }
        } else {
            if device.supports_color {
                lines.push(Line::from(Span::raw("🎨 Loading...")));
            }
            if device.supports_color_temp {
                lines.push(Line::from(Span::raw("🌡️ Loading...")));
            }
        }

        let color_widget = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Color & Temperature"),
        );
        frame.render_widget(color_widget, chunks[3]);
    } else {
        let color_widget = Paragraph::new(Line::from(Span::styled("Not supported", theme.dim)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Color & Temperature"),
            );
        frame.render_widget(color_widget, chunks[3]);
    }

    // Controls help - compact format
    let mut help_parts = vec![];

    if device.supports_power {
        help_parts.push("[Space] Power");
    }
    if device.supports_brightness {
        help_parts.push("[↑↓] Bright");
        help_parts.push("[1-9] Quick%");
    }
    if device.supports_color {
        help_parts.push("[C] Color");
    }
    if device.supports_color_temp {
        help_parts.push("[t/T] Temp");
    }
    help_parts.push("[Esc] Back");

    let help_text = help_parts.join("  ");
    let help = Paragraph::new(Line::from(Span::styled(&help_text, theme.dim)))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(help, chunks[4]);
}
