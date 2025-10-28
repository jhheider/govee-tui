use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::App;
use super::theme::Emoji;
use super::view_state::ViewMode;
use super::widgets;
use crate::api::Device;

impl App {
    pub fn render(&self, frame: &mut Frame) {
        match &self.state.view_mode {
            ViewMode::List => self.render_list(frame),
            ViewMode::Panel => self.render_panel(frame),
            ViewMode::Detail => self.render_detail(frame),
            ViewMode::Brightness => self.render_brightness(frame),
            ViewMode::ColorPicker => self.render_color_picker(frame),
            ViewMode::Search => self.render_search(frame),
            ViewMode::Help => self.render_help(frame),
        }
    }

    fn render_list(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Title
        let title_text = if self.state.selected_devices.is_empty() {
            format!(
                "{} Govee Controller - {} devices",
                Emoji::HOME,
                self.devices.len()
            )
        } else {
            format!(
                "{} Govee Controller - {} selected of {}",
                Emoji::HOME,
                self.state.selected_devices.len(),
                self.devices.len()
            )
        };

        let title = Paragraph::new(title_text).style(self.theme.title).block(
            Block::default()
                .borders(Borders::ALL)
                .style(self.theme.border),
        );
        frame.render_widget(title, chunks[0]);

        // Device list with selection indicators
        let device_list = widgets::device_list::render(
            &self.devices,
            self.state.selected_index,
            &self.state.selected_devices,
            &self.theme,
        );
        frame.render_widget(device_list, chunks[1]);

        // Status bar with contextual controls
        let status_text = if self.loading {
            format!("{} Loading...", Emoji::LOADING)
        } else if let Some(msg) = &self.state.status_message {
            msg.to_string()
        } else {
            "[↑↓] Navigate  [Enter] Details  [Space] Select  [Tab] Panel  [R]efresh  [?] Help  [Q]uit".to_string()
        };

        let status = Paragraph::new(status_text).style(self.theme.dim).block(
            Block::default()
                .borders(Borders::ALL)
                .style(self.theme.border),
        );
        frame.render_widget(status, chunks[2]);
    }

    fn render_panel(&self, frame: &mut Frame) {
        widgets::panel_view::render(
            &self.devices,
            &self.state.all_device_states,
            self.state.selected_index,
            &self.theme,
            frame,
        );
    }

    fn render_detail(&self, frame: &mut Frame) {
        if let Some(device) = self.selected_device() {
            widgets::detail_view::render(
                device,
                self.state.device_state.as_ref(),
                &self.theme,
                frame,
            );
        }
    }

    fn render_brightness(&self, frame: &mut Frame) {
        if let Some(brightness) = &self.state.brightness_control {
            widgets::brightness::render(brightness, &self.theme, frame);
        }
    }

    fn render_color_picker(&self, frame: &mut Frame) {
        if let Some(picker) = &self.state.color_picker {
            widgets::color_picker::render(picker, &self.theme, frame);
        }
    }

    fn render_search(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        let search_box = Paragraph::new(format!(
            "{} Search: {}",
            Emoji::SEARCH,
            self.state.search_query
        ))
        .style(self.theme.title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Device Search"),
        );
        frame.render_widget(search_box, chunks[0]);

        // Filter devices (pre-lowercase query to avoid repeated allocations)
        let query_lower = self.state.search_query.to_lowercase();
        let filtered: Vec<Device> = self
            .devices
            .iter()
            .filter(|d| {
                query_lower.is_empty()
                    || d.name.to_lowercase().contains(&query_lower)
                    || d.model.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        let empty_selection = vec![];
        let list = widgets::device_list::render(&filtered, 0, &empty_selection, &self.theme);
        frame.render_widget(list, chunks[1]);
    }

    fn render_help(&self, frame: &mut Frame) {
        let help_text = vec![
            format!("{} Govee TUI - Keyboard Shortcuts", Emoji::HELP),
            "".to_string(),
            "═══ GENERAL ═══".to_string(),
            "  q, Esc       Quit / Go back".to_string(),
            "  ?            Show this help".to_string(),
            "  r            Refresh devices".to_string(),
            "".to_string(),
            "═══ DEVICE LIST ═══".to_string(),
            "  ↑/↓, j/k     Navigate devices".to_string(),
            "  Enter        View device details".to_string(),
            "  Space        Toggle multi-select".to_string(),
            "  Tab          Switch to Panel view".to_string(),
            "  Ctrl+F       Search devices".to_string(),
            "".to_string(),
            "═══ PANEL VIEW ═══".to_string(),
            "  ↑/↓, j/k     Navigate devices".to_string(),
            "  Enter        View device details".to_string(),
            "  Space        Toggle power ON/OFF".to_string(),
            "  Tab          Switch to List view".to_string(),
            "  r            Refresh all states".to_string(),
            "".to_string(),
            "═══ DETAIL VIEW ═══".to_string(),
            "  Space        Toggle power ON/OFF".to_string(),
            "  ↑/↓, j/k     Brightness ±10%".to_string(),
            "  Shift+↑/↓    Brightness ±5% (fine)".to_string(),
            "  c            Change color (RGB)".to_string(),
            "  Esc          Back to list".to_string(),
            "".to_string(),
            "═══ BRIGHTNESS CONTROL ═══".to_string(),
            "  ↑/↓, j/k     Adjust by 10%".to_string(),
            "  Shift+↑/↓    Adjust by 5% (fine)".to_string(),
            "  1-9          Set to 10%-90%".to_string(),
            "  Enter        Apply changes".to_string(),
            "  Esc          Cancel".to_string(),
            "".to_string(),
            "═══ COLOR PICKER ═══".to_string(),
            "  Tab          Switch R/G/B channel".to_string(),
            "  ↑/↓, j/k     Adjust by 10".to_string(),
            "  Shift+↑/↓    Adjust by 5 (fine)".to_string(),
            "  Enter        Apply color".to_string(),
            "  Esc          Cancel".to_string(),
            "".to_string(),
            "═══ SEARCH ═══".to_string(),
            "  Type         Filter devices".to_string(),
            "  Backspace    Delete character".to_string(),
            "  Esc          Back to list".to_string(),
            "".to_string(),
            "Press any key to close help...".to_string(),
        ];

        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .style(self.theme.text)
            .block(
                Block::default()
                    .title(format!("{} Keyboard Shortcuts", Emoji::HELP))
                    .borders(Borders::ALL)
                    .style(self.theme.border),
            );

        frame.render_widget(help_paragraph, frame.area());
    }
}
