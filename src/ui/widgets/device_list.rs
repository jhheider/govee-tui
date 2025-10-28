use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::api::Device;
use crate::ui::theme::{Emoji, Theme};

pub fn render<'a>(
    devices: &'a [Device],
    selected_index: usize,
    selected_devices: &'a [usize],
    theme: &'a Theme,
) -> List<'a> {
    let items: Vec<ListItem> = devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            // Show controllable status (we don't have power state in list view)
            let status_emoji = if device.controllable {
                Emoji::SUCCESS
            } else {
                "⚪"
            };

            let device_emoji = if device.is_group {
                "📦" // Group emoji
            } else {
                Emoji::LIGHT
            };

            // Multi-select indicator
            let select_indicator = if selected_devices.contains(&i) {
                "[✓] "
            } else {
                "[ ] "
            };

            let content = Line::from(vec![
                Span::raw(select_indicator),
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
                .style(theme.border),
        )
        .highlight_style(theme.highlight)
}
