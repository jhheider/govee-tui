use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{self, Emoji, Theme};

pub fn render_with_style(
    device: &Device,
    state: Option<&DeviceState>,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    border_style: Style,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    // Device header with emoji
    let device_emoji = if device.is_group { "📦" } else { Emoji::LIGHT };
    let header = Paragraph::new(format!(
        "{} {} ({})",
        device_emoji,
        device.name,
        device.model
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
    if device.supports_power { caps.push("⚡Power"); }
    if device.supports_brightness { caps.push("☀️Bright"); }
    if device.supports_color { caps.push("🎨Color"); }
    if device.supports_color_temp { caps.push("🌡️Temp"); }
    let caps_str = if caps.is_empty() {
        "None".to_string()
    } else {
        caps.join(" ")
    };

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Power: "),
            Span::styled(&power_status, theme.text),
        ]),
        Line::from(vec![
            Span::raw("Capabilities: "),
            Span::styled(&caps_str, theme.highlight),
        ]),
        Line::from(vec![
            Span::raw("Model: "),
            Span::styled(&device.model, theme.dim),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Info"));
    frame.render_widget(info, chunks[1]);

    // Brightness - always show if device supports it
    if device.supports_brightness {
        let (brightness_text, bar_text) = if let Some(s) = state {
            if let Some(brightness) = s.brightness {
                let bar = theme::brightness_bar(brightness as u8, 20);
                (format!("{} Brightness: {}%", Emoji::BRIGHTNESS, brightness), bar)
            } else {
                (format!("{} Brightness: Unknown", Emoji::BRIGHTNESS), String::new())
            }
        } else {
            (format!("{} Brightness: Loading...", Emoji::BRIGHTNESS), String::new())
        };

        let brightness_widget = Paragraph::new(vec![
            Line::from(brightness_text),
            Line::from(bar_text),
        ])
        .block(Block::default().borders(Borders::ALL).title("Brightness"));
        frame.render_widget(brightness_widget, chunks[2]);
    } else {
        let brightness_widget = Paragraph::new(vec![
            Line::from("Not supported by this device"),
        ])
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).title("Brightness"));
        frame.render_widget(brightness_widget, chunks[2]);
    }

    // Color/Temperature - always show if device supports it
    if device.supports_color || device.supports_color_temp {
        let mut lines = vec![];

        if let Some(s) = state {
            if let Some(color) = s.color {
                lines.push(Line::from(theme::color_indicator(
                    color.r, color.g, color.b,
                )));
            } else if device.supports_color {
                lines.push(Line::from("RGB: Loading..."));
            }

            if let Some(temp) = s.color_temp {
                lines.push(Line::from(theme::temp_indicator(temp)));
            } else if device.supports_color_temp {
                lines.push(Line::from("Temperature: Loading..."));
            }
        } else {
            if device.supports_color {
                lines.push(Line::from("RGB: Loading..."));
            }
            if device.supports_color_temp {
                lines.push(Line::from("Temperature: Loading..."));
            }
        }

        let color_widget =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Color"));
        frame.render_widget(color_widget, chunks[3]);
    } else {
        let color_widget = Paragraph::new(vec![
            Line::from("Not supported by this device"),
        ])
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).title("Color"));
        frame.render_widget(color_widget, chunks[3]);
    }

    // Controls help - show what's actually available
    let mut help_lines = vec![];

    if device.supports_power {
        help_lines.push(Line::from(vec![
            Span::styled("[Space]", theme.highlight),
            Span::raw(" Power On/Off"),
        ]));
    }

    if device.supports_brightness {
        help_lines.push(Line::from(vec![
            Span::styled("[↑↓]", theme.highlight),
            Span::raw(" Brightness ±10%  "),
            Span::styled("[Shift+↑↓]", theme.highlight),
            Span::raw(" ±5%"),
        ]));
    }

    if device.supports_color {
        help_lines.push(Line::from(vec![
            Span::styled("[C]", theme.highlight),
            Span::raw(" Color Picker ("),
            Span::styled("Enter", theme.highlight),
            Span::raw(" to apply)"),
        ]));
    }

    help_lines.push(Line::from(vec![
        Span::styled("[Esc]", theme.highlight),
        Span::raw(" Back to List  "),
        Span::styled("[Tab]", theme.highlight),
        Span::raw(" Switch Focus"),
    ]));

    let help = Paragraph::new(help_lines)
        .style(theme.text)
        .block(Block::default().borders(Borders::ALL).title("Available Controls"));
    frame.render_widget(help, chunks[4]);
}
