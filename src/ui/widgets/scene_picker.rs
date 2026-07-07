use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::Scene;
use crate::ui::theme::{Emoji, Theme};

/// Modal for browsing and applying a device's light scenes.
/// `scenes: None` means the list is still loading.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenePicker {
    pub device_name: String,
    pub scenes: Option<Vec<Scene>>,
    pub selected: usize,
}

impl ScenePicker {
    pub fn loading(device_name: String) -> Self {
        Self {
            device_name,
            scenes: None,
            selected: 0,
        }
    }

    pub fn with_scenes(device_name: String, scenes: Vec<Scene>) -> Self {
        Self {
            device_name,
            scenes: Some(scenes),
            selected: 0,
        }
    }

    pub fn next(&mut self) {
        if let Some(scenes) = &self.scenes {
            if !scenes.is_empty() {
                self.selected = (self.selected + 1) % scenes.len();
            }
        }
    }

    pub fn prev(&mut self) {
        if let Some(scenes) = &self.scenes {
            if !scenes.is_empty() {
                self.selected = self.selected.checked_sub(1).unwrap_or(scenes.len() - 1);
            }
        }
    }

    pub fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(10);
    }

    pub fn page_down(&mut self) {
        if let Some(scenes) = &self.scenes {
            if !scenes.is_empty() {
                self.selected = (self.selected + 10).min(scenes.len() - 1);
            }
        }
    }

    pub fn home(&mut self) {
        self.selected = 0;
    }

    pub fn end(&mut self) {
        if let Some(scenes) = &self.scenes {
            self.selected = scenes.len().saturating_sub(1);
        }
    }

    pub fn selected_scene(&self) -> Option<&Scene> {
        self.scenes.as_ref()?.get(self.selected)
    }
}

pub fn render(picker: &ScenePicker, theme: &Theme, frame: &mut Frame) {
    let area = frame.area();
    let popup = Rect {
        x: area.width / 5,
        y: area.height / 8,
        width: area.width * 3 / 5,
        height: area.height * 3 / 4,
    };
    frame.render_widget(Clear, popup);

    let title = match &picker.scenes {
        Some(scenes) if !scenes.is_empty() => {
            format!(" 🎬 Scenes - {} ({}) ", picker.device_name, scenes.len())
        }
        _ => format!(" 🎬 Scenes - {} ", picker.device_name),
    };

    match &picker.scenes {
        None => {
            let loading = Paragraph::new(vec![
                Line::from(""),
                Line::from(format!("{} Loading scenes…", Emoji::LOADING)),
            ])
            .style(theme.text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(theme.border_focused),
            );
            frame.render_widget(loading, popup);
        }
        Some(scenes) if scenes.is_empty() => {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from("No scenes available for this device"),
                Line::from(""),
                Line::from(Span::styled("[esc] close", theme.dim)),
            ])
            .style(theme.text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(theme.border_focused),
            );
            frame.render_widget(empty, popup);
        }
        Some(scenes) => {
            let items: Vec<ListItem> = scenes
                .iter()
                .map(|scene| {
                    let mut spans = vec![Span::raw(scene.name.clone())];
                    if scene.param_id.is_none() {
                        spans.push(Span::styled("  (DIY)", theme.dim));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(title)
                        .title_bottom(
                            " [↑↓/jk] browse  [pgup/pgdn] page  [enter] apply  [esc] close ",
                        )
                        .borders(Borders::ALL)
                        .border_style(theme.border_focused),
                )
                .style(theme.text)
                .highlight_style(theme.highlight)
                .highlight_symbol("▶ ");

            let mut state = ListState::default();
            state.select(Some(picker.selected));
            frame.render_stateful_widget(list, popup, &mut state);
        }
    }
}
