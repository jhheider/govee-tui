use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{Emoji, Theme};

pub fn render(
    devices: &[Device],
    states: &[Option<DeviceState>],
    selected_index: usize,
    theme: &Theme,
    frame: &mut Frame,
) {
    let area = frame.area();

    // Build table header
    let header = Row::new(vec![
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Model").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Power").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Brightness").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Online").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    // Build table rows
    let rows: Vec<Row> = devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let type_icon = if device.is_group {
                "📦"
            } else {
                Emoji::LIGHT
            };

            let state = states.get(i).and_then(|s| s.as_ref());

            let power_status = if let Some(s) = state {
                if s.power {
                    format!("{} ON ", Emoji::POWER_ON)
                } else {
                    format!("{} OFF", Emoji::POWER_OFF)
                }
            } else {
                "  -  ".to_string()
            };

            let brightness = if let Some(s) = state {
                if let Some(b) = s.brightness {
                    format!("{}%", b)
                } else {
                    "  -  ".to_string()
                }
            } else {
                "  -  ".to_string()
            };

            let online_status = if let Some(s) = state {
                if s.online {
                    format!("{} ", Emoji::SUCCESS)
                } else {
                    "⚠️ ".to_string()
                }
            } else {
                "  -  ".to_string()
            };

            let row_style = if i == selected_index {
                theme.highlight
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(type_icon),
                Cell::from(device.name.clone()),
                Cell::from(device.model.clone()),
                Cell::from(power_status),
                Cell::from(brightness),
                Cell::from(online_status),
            ])
            .style(row_style)
        })
        .collect();

    // Calculate column widths
    let widths = [
        Constraint::Length(4),      // Type
        Constraint::Percentage(30), // Name
        Constraint::Percentage(25), // Model
        Constraint::Length(8),      // Power
        Constraint::Length(10),     // Brightness
        Constraint::Length(6),      // Online
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!("{} All Devices (Panel View)", Emoji::DEVICE))
                .borders(Borders::ALL)
                .style(theme.border),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(table, area);

    // Status bar at the bottom
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(3),
        width: area.width,
        height: 3,
    };

    let help_text =
        "[Space] Toggle  [↑↓] Navigate  [Enter] Detail  [Tab] List View  [R] Refresh  [Q] Quit";
    let help_widget = Block::default()
        .borders(Borders::ALL)
        .title("Controls")
        .style(theme.dim);

    frame.render_widget(help_widget, help_area);
    frame.render_widget(
        Line::from(help_text).style(theme.dim),
        Rect {
            x: help_area.x + 2,
            y: help_area.y + 1,
            width: help_area.width.saturating_sub(4),
            height: 1,
        },
    );
}
