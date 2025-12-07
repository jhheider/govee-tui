use std::collections::HashSet;

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::api::Device;
use crate::ui::focus::Focus;
use crate::ui::theme::{Emoji, Theme};

pub fn render_with_style<'a>(
    devices: &'a [Device],
    selected_index: usize,
    selected_devices: &'a HashSet<String>,
    focus: &Focus,
    search_query: &str,
    theme: &'a Theme,
    border_style: Style,
) -> List<'a> {
    let items: Vec<ListItem> = devices
        .iter()
        .enumerate()
        .filter(|(_, device)| {
            if search_query.is_empty() {
                true
            } else {
                let query = search_query.to_lowercase();
                device.name.to_lowercase().contains(&query)
                    || device.model.to_lowercase().contains(&query)
            }
        })
        .map(|(i, device)| {
            // Checkbox for multi-select
            let checkbox = if selected_devices.contains(&device.id) {
                "[✓] "
            } else {
                "[ ] "
            };

            // Device type emoji
            let device_emoji = if device.is_group {
                "📦"
            } else {
                Emoji::LIGHT
            };

            // Power/online status
            let status = if !device.controllable {
                "⚫" // Offline/uncontrollable
            } else {
                "🟢" // Online (we don't have power state in list)
            };

            let content = Line::from(vec![
                Span::raw(checkbox),
                Span::raw(device_emoji),
                Span::raw(" "),
                Span::styled(&device.name, theme.text),
                Span::raw(" "),
                Span::raw(status),
            ]);

            let mut item = ListItem::new(content);

            if i == selected_index {
                item = item.style(theme.highlight);
            }

            item
        })
        .collect();

    // Build title with focus indicator and selection count
    let focus_indicator = if *focus == Focus::List { " ●" } else { "" };
    let selection_info = if selected_devices.is_empty() {
        String::new()
    } else {
        format!(" ({} selected)", selected_devices.len())
    };
    let search_info = if !search_query.is_empty() {
        format!(" [{}]", search_query)
    } else {
        String::new()
    };

    let title = format!(
        "{} Devices{}{}{}",
        Emoji::DEVICE,
        focus_indicator,
        selection_info,
        search_info
    );

    List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(theme.highlight)
}
