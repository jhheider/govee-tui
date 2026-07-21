use color_name::css::Color as ColorName;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::color_groups::{get_color_groups, to_spaced_name};
use crate::ui::theme::{Emoji, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorPickerMode {
    Rgb,     // Edit RGB values
    Browser, // Browse named colors
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPicker {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub mode: ColorPickerMode,
    pub selected_channel: usize, // 0=R, 1=G, 2=B (RGB mode)
    pub selected_group: usize,   // Which color group (Browser mode)
    pub selected_color: usize,   // Which color in group (Browser mode)
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
    use ratatui::{layout::Rect, widgets::Clear};

    // Centered popup over the main view
    let screen = frame.area();
    let width = screen.width.min(72);
    let height = if picker.mode == ColorPickerMode::Browser {
        screen.height.min(28)
    } else {
        screen.height.min(16)
    };
    let area = Rect {
        x: screen.width.saturating_sub(width) / 2,
        y: screen.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, area);

    // Different layout based on mode
    let chunks = if picker.mode == ColorPickerMode::Browser {
        // Browser mode: Title | Group selector | Preview | Color list | Help
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Group selector
                Constraint::Length(4), // Preview
                Constraint::Min(8),    // Color list
                Constraint::Length(3), // Help
            ])
            .split(area)
    } else {
        // RGB mode: Title | Preview | Sliders | Help
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(4), // Preview
                Constraint::Length(5), // RGB sliders
                Constraint::Length(3), // Help
            ])
            .split(area)
    };

    // Title
    let mode_str = match picker.mode {
        ColorPickerMode::Rgb => "RGB Editor",
        ColorPickerMode::Browser => "Color Browser",
    };
    let title = Paragraph::new(format!("{} Color Picker - {}", Emoji::COLOR, mode_str))
        .style(theme.title)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Group selector (Browser mode only)
    if picker.mode == ColorPickerMode::Browser {
        let groups = get_color_groups();
        let mut columns = vec![];

        for (i, group) in groups.iter().enumerate() {
            columns.push(Span::raw("  "));
            let style = if i == picker.selected_group {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(ratatui::style::Color::Cyan)
            } else {
                Style::default()
            };
            columns.push(Span::styled(
                format!("{} {}", group.emoji, group.name),
                style,
            ));
        }

        let selector = Paragraph::new(Line::from(columns))
            .block(Block::default().borders(Borders::ALL).title("Groups"));
        frame.render_widget(selector, chunks[1]);
    }

    // Color preview with name lookup
    let color_name = ColorName::similar([picker.r, picker.g, picker.b]);
    let spaced_name = to_spaced_name(&color_name);

    let preview_chunk = if picker.mode == ColorPickerMode::Browser {
        chunks[2]
    } else {
        chunks[1]
    };

    let preview = Paragraph::new(vec![
        Line::from(vec![
            crate::ui::theme::color_swatch(picker.r, picker.g, picker.b, 24),
            Span::raw(format!("  {spaced_name}")),
        ]),
        Line::from(format!(
            "RGB({:3},{:3},{:3})  #{:02X}{:02X}{:02X}",
            picker.r, picker.g, picker.b, picker.r, picker.g, picker.b
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("Preview"));
    frame.render_widget(preview, preview_chunk);

    // Main content area: either RGB sliders or color browser
    let main_chunk = if picker.mode == ColorPickerMode::Browser {
        chunks[3]
    } else {
        chunks[2]
    };

    match picker.mode {
        ColorPickerMode::Rgb => {
            // RGB sliders
            let r_style = if picker.selected_channel == 0 {
                theme.highlight
            } else {
                theme.text
            };
            let g_style = if picker.selected_channel == 1 {
                theme.highlight
            } else {
                theme.text
            };
            let b_style = if picker.selected_channel == 2 {
                theme.highlight
            } else {
                theme.text
            };

            use ratatui::style::Color;
            let bar = |value: u8, color: Color| {
                Span::styled(
                    "█".repeat((value as usize * 20) / 255),
                    Style::default().fg(color),
                )
            };
            let sliders = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Red:   ", r_style),
                    Span::raw(format!("{:3} ", picker.r)),
                    bar(picker.r, Color::Red),
                ]),
                Line::from(vec![
                    Span::styled("Green: ", g_style),
                    Span::raw(format!("{:3} ", picker.g)),
                    bar(picker.g, Color::Green),
                ]),
                Line::from(vec![
                    Span::styled("Blue:  ", b_style),
                    Span::raw(format!("{:3} ", picker.b)),
                    bar(picker.b, Color::Blue),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title("RGB Channels"));
            frame.render_widget(sliders, main_chunk);
        }
        ColorPickerMode::Browser => {
            // Color browser: just show color names with simple swatches
            let groups = get_color_groups();
            if let Some(group) = groups.get(picker.selected_group) {
                let items: Vec<ListItem> = group
                    .colors
                    .iter()
                    .enumerate()
                    .map(|(i, (name, rgb))| {
                        let spaced = to_spaced_name(name);
                        let is_selected = i == picker.selected_color;
                        let style = if is_selected {
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(ratatui::style::Color::Cyan)
                        } else {
                            Style::default()
                        };

                        ListItem::new(Line::from(vec![
                            crate::ui::theme::color_swatch(rgb[0], rgb[1], rgb[2], 3),
                            Span::styled("  ", style),
                            Span::styled(format!("{spaced:<25}"), style),
                            Span::styled(
                                format!("RGB({:3},{:3},{:3})", rgb[0], rgb[1], rgb[2]),
                                if is_selected {
                                    theme.dim
                                } else {
                                    Style::default().fg(ratatui::style::Color::DarkGray)
                                },
                            ),
                        ]))
                    })
                    .collect();

                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} {} Colors", group.emoji, group.name)),
                );
                frame.render_widget(list, main_chunk);
            }
        }
    }

    // Help text
    let help_chunk = if picker.mode == ColorPickerMode::Browser {
        chunks[4]
    } else {
        chunks[3]
    };

    let help_text = match picker.mode {
        ColorPickerMode::Rgb => {
            "[↑↓/jk] channel [←→/hl] ±10 [tab] browser [enter] apply [esc] cancel"
        }
        ColorPickerMode::Browser => {
            "[↑↓/jk] colors [←→/hl] groups [tab] rgb [enter] apply [esc] cancel"
        }
    };

    let help = Paragraph::new(help_text)
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(help, help_chunk);
}
