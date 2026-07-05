use std::collections::HashMap;

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::api::{models::DeviceState, Device};
use crate::ui::theme::{Emoji, Theme};

pub fn render_with_style<'a>(
    devices: &'a [Device],
    known_states: &HashMap<String, DeviceState>,
    selected_index: usize,
    theme: &'a Theme,
    border_style: Style,
) -> List<'a> {
    let items: Vec<ListItem> = devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
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

            let content = Line::from(vec![
                Span::raw(device_emoji),
                Span::raw(" "),
                Span::styled(&device.name, theme.text),
                Span::raw(" "),
                Span::raw(status_emoji),
                Span::raw("  "),
                Span::styled(&device.model, theme.dim),
            ]);

            let mut item = ListItem::new(content);

            if i == selected_index {
                item = item.style(theme.highlight);
            }

            item
        })
        .collect();

    List::new(items)
        .block(
            Block::default()
                .title(format!("{} Devices", Emoji::DEVICE))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(theme.highlight)
}
