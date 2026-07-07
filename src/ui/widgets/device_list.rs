use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{Emoji, Theme};

#[allow(clippy::too_many_arguments)]
pub fn render_with_style(
    devices: &[Device],
    known_states: &HashMap<String, DeviceState>,
    selected_index: usize,
    loading: bool,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    border_style: Style,
) {
    let block = Block::default()
        .title(format!("{} Devices", Emoji::DEVICE))
        .borders(Borders::ALL)
        .border_style(border_style);

    if devices.is_empty() {
        let placeholder = if loading {
            format!("{} Loading devices…", Emoji::LOADING)
        } else {
            "No devices found - press r to refresh".to_string()
        };
        let empty = Paragraph::new(Line::from(placeholder))
            .style(theme.dim)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = devices
        .iter()
        .map(|device| {
            let device_emoji = if device.is_group {
                "📦" // Group emoji
            } else {
                Emoji::LIGHT
            };

            // Last confirmed power state, if we've seen one
            let power = known_states.get(&device.id).map(|s| s.power);
            let status_emoji = match (power, device.controllable) {
                (Some(true), _) => Emoji::POWER_ON,
                (Some(false), _) => Emoji::POWER_OFF,
                (None, true) => "·",
                (None, false) => "⚪",
            };

            ListItem::new(Line::from(vec![
                Span::raw(device_emoji),
                Span::raw(" "),
                Span::raw(&device.name),
                Span::raw(" "),
                Span::raw(status_emoji),
                Span::raw("  "),
                Span::styled(&device.model, theme.dim),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(theme.text)
        .highlight_style(theme.highlight)
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(selected_index));
    frame.render_stateful_widget(list, area, &mut state);
}
