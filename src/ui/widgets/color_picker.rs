use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame,
    style::{Style, Modifier},
};
use color_name::css::Color as ColorName;

use crate::ui::theme::{Emoji, Theme};
use super::color_groups::{get_color_groups, to_spaced_name};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorPickerMode {
    Rgb,      // Edit RGB values
    Browser,  // Browse named colors
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPicker {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub mode: ColorPickerMode,
    pub selected_channel: usize, // 0=R, 1=G, 2=B (RGB mode)
    pub selected_group: usize,    // Which color group (Browser mode)
    pub selected_color: usize,    // Which color in group (Browser mode)
}

impl ColorPicker {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            mode: ColorPickerMode::Rgb,
            selected_channel: 0,
            selected_group: 0,
            selected_color: 0,
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ColorPickerMode::Rgb => ColorPickerMode::Browser,
            ColorPickerMode::Browser => ColorPickerMode::Rgb,
        };
    }

    pub fn adjust(&mut self, delta: i16) {
        let current = match self.selected_channel {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            _ => return,
        };

        *current = (*current as i16 + delta).clamp(0, 255) as u8;
    }

    pub fn next_channel(&mut self) {
        self.selected_channel = (self.selected_channel + 1) % 3;
    }

    pub fn prev_channel(&mut self) {
        self.selected_channel = if self.selected_channel == 0 {
            2
        } else {
            self.selected_channel - 1
        };
    }

    pub fn next_group(&mut self) {
        let groups = get_color_groups();
        self.selected_group = (self.selected_group + 1) % groups.len();
        self.selected_color = 0; // Reset to first color in new group
    }

    pub fn prev_group(&mut self) {
        let groups = get_color_groups();
        self.selected_group = if self.selected_group == 0 {
            groups.len() - 1
        } else {
            self.selected_group - 1
        };
        self.selected_color = 0;
    }

    pub fn next_color(&mut self) {
        let groups = get_color_groups();
        if let Some(group) = groups.get(self.selected_group) {
            self.selected_color = (self.selected_color + 1) % group.colors.len();
        }
    }

    pub fn prev_color(&mut self) {
        let groups = get_color_groups();
        if let Some(group) = groups.get(self.selected_group) {
            self.selected_color = if self.selected_color == 0 {
                group.colors.len() - 1
            } else {
                self.selected_color - 1
            };
        }
    }

    pub fn select_current_color(&mut self) {
        let groups = get_color_groups();
        if let Some(group) = groups.get(self.selected_group) {
            if let Some((_, rgb)) = group.colors.get(self.selected_color) {
                self.r = rgb[0];
                self.g = rgb[1];
                self.b = rgb[2];
            }
        }
    }
}

pub fn render(picker: &ColorPicker, theme: &Theme, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Length(10),  // Preview
            Constraint::Min(10),     // RGB sliders OR color browser
            Constraint::Length(4),   // Help
        ])
        .split(area);

    // Title shows current mode
    let mode_str = match picker.mode {
        ColorPickerMode::Rgb => "RGB Editor",
        ColorPickerMode::Browser => "Color Browser",
    };
    let title = Paragraph::new(format!("{} Color Picker - {}", Emoji::COLOR, mode_str))
        .style(theme.title)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Color preview with name lookup
    let color_name = ColorName::similar([picker.r, picker.g, picker.b]);
    let spaced_name = to_spaced_name(&color_name);

    let preview = Paragraph::new(vec![
        Line::from(format!(
            "Current: {}",
            crate::ui::theme::color_indicator(picker.r, picker.g, picker.b)
        )),
        Line::from(format!(
            "Name: {}",
            spaced_name
        )),
        Line::from(format!(
            "RGB: ({}, {}, {})",
            picker.r, picker.g, picker.b
        )),
        Line::from(format!(
            "Hex: #{:02X}{:02X}{:02X}",
            picker.r, picker.g, picker.b
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("Preview"));
    frame.render_widget(preview, chunks[1]);

    // Main content area - either RGB sliders or color browser
    match picker.mode {
        ColorPickerMode::Rgb => {
            // RGB sliders
            let r_style = if picker.selected_channel == 0 { theme.highlight } else { theme.text };
            let g_style = if picker.selected_channel == 1 { theme.highlight } else { theme.text };
            let b_style = if picker.selected_channel == 2 { theme.highlight } else { theme.text };

            let sliders = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("🔴 Red:   ", r_style),
                    Span::raw(format!("{:3} ", picker.r)),
                    Span::raw("█".repeat((picker.r as usize * 20) / 255)),
                ]),
                Line::from(vec![
                    Span::styled("🟢 Green: ", g_style),
                    Span::raw(format!("{:3} ", picker.g)),
                    Span::raw("█".repeat((picker.g as usize * 20) / 255)),
                ]),
                Line::from(vec![
                    Span::styled("🔵 Blue:  ", b_style),
                    Span::raw(format!("{:3} ", picker.b)),
                    Span::raw("█".repeat((picker.b as usize * 20) / 255)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title("RGB Channels"));
            frame.render_widget(sliders, chunks[2]);
        }
        ColorPickerMode::Browser => {
            // Color browser
            let groups = get_color_groups();
            if let Some(group) = groups.get(picker.selected_group) {
                let items: Vec<ListItem> = group.colors.iter().enumerate().map(|(i, (name, rgb))| {
                    let spaced = to_spaced_name(name);
                    let is_selected = i == picker.selected_color;
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD).fg(ratatui::style::Color::Cyan)
                    } else {
                        Style::default()
                    };

                    let color_swatch = crate::ui::theme::color_indicator(rgb[0], rgb[1], rgb[2]);
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{} ", color_swatch), style),
                        Span::styled(spaced, style),
                    ]))
                }).collect();

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} {} Colors", group.emoji, group.name)));
                frame.render_widget(list, chunks[2]);
            }
        }
    }

    // Help text changes based on mode
    let help_text = match picker.mode {
        ColorPickerMode::Rgb => "[↑↓] Switch Channel  [←→] Adjust ±10  [Tab] Color Browser  [Enter] Apply  [Esc] Cancel",
        ColorPickerMode::Browser => "[↑↓] Navigate Colors  [←→] Switch Group  [Enter] Select  [Tab] RGB Mode  [Esc] Cancel",
    };

    let help = Paragraph::new(help_text)
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(help, chunks[3]);
}
