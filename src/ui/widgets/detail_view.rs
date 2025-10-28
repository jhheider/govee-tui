use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{self, Emoji, Theme};

pub fn render(device: &Device, state: Option<&DeviceState>, theme: &Theme, frame: &mut Frame) {
    let area = frame.area();

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

    // Device header
    let header = Paragraph::new(format!(
        "{} {} ({})",
        Emoji::LIGHT,
        device.name,
        device.model
    ))
    .style(theme.title)
    .block(Block::default().borders(Borders::ALL).style(theme.border));
    frame.render_widget(header, chunks[0]);

    // Power and basic info
    let power_status = if let Some(s) = state {
        if s.power {
            format!("{} ON", Emoji::POWER_ON)
        } else {
            format!("{} OFF", Emoji::POWER_OFF)
        }
    } else {
        "Unknown".to_string()
    };

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Device ID: "),
            Span::styled(&device.id, theme.dim),
        ]),
        Line::from(vec![
            Span::raw("Power: "),
            Span::styled(&power_status, theme.text),
        ]),
        Line::from(vec![
            Span::raw("Controllable: "),
            Span::styled(
                if device.controllable { "Yes" } else { "No" },
                if device.controllable {
                    theme.success
                } else {
                    theme.dim
                },
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Info"));
    frame.render_widget(info, chunks[1]);

    // Brightness
    if let Some(s) = state {
        if let Some(brightness) = s.brightness {
            let bar = theme::brightness_bar(brightness, 20);
            let brightness_widget = Paragraph::new(vec![
                Line::from(format!("{} Brightness: {}%", Emoji::BRIGHTNESS, brightness)),
                Line::from(bar),
            ])
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(brightness_widget, chunks[2]);
        }
    }

    // Color/Temperature
    if let Some(s) = state {
        let mut lines = vec![];
        if let Some(color) = s.color {
            lines.push(Line::from(theme::color_indicator(
                color.r, color.g, color.b,
            )));
        }
        if let Some(temp) = s.color_temp {
            lines.push(Line::from(theme::temp_indicator(temp)));
        }

        if !lines.is_empty() {
            let color_widget =
                Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Color"));
            frame.render_widget(color_widget, chunks[3]);
        }
    }

    // Controls help - contextual status bar with more prominent styling
    let help_lines = vec![
        Line::from(vec![
            Span::styled("[Space]", theme.highlight),
            Span::raw(" Power On/Off  "),
            Span::styled("[↑↓]", theme.highlight),
            Span::raw(" Brightness ±10%  "),
            Span::styled("[Shift+↑↓]", theme.highlight),
            Span::raw(" ±5%  "),
        ]),
        Line::from(vec![
            Span::styled("[C]", theme.highlight),
            Span::raw(" Color Picker  "),
            Span::styled("[Esc]", theme.highlight),
            Span::raw(" Back to List  "),
            Span::styled("[?]", theme.highlight),
            Span::raw(" Help"),
        ]),
    ];
    let help = Paragraph::new(help_lines)
        .style(theme.text)
        .block(Block::default().borders(Borders::ALL).title("Interactive Controls"));
    frame.render_widget(help, chunks[4]);
}
