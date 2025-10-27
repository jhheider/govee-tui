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
            ViewMode::Detail => self.render_detail(frame),
            ViewMode::Brightness => self.render_brightness(frame),
            ViewMode::ColorPicker => self.render_color_picker(frame),
            ViewMode::Search => self.render_search(frame),
        }
    }

    fn render_list(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title
        let title_text = if self.state.selected_devices.is_empty() {
            format!("{} Govee Controller - {} devices", Emoji::HOME, self.devices.len())
        } else {
            format!(
                "{} Govee Controller - {} selected of {}",
                Emoji::HOME,
                self.state.selected_devices.len(),
                self.devices.len()
            )
        };

        let title = Paragraph::new(title_text)
            .style(self.theme.title)
            .block(Block::default().borders(Borders::ALL).style(self.theme.border));
        frame.render_widget(title, chunks[0]);

        // Device list with selection indicators
        let device_list = widgets::device_list::render(
            &self.devices,
            self.state.selected_index,
            &self.state.selected_devices,
            &self.theme,
        );
        frame.render_widget(device_list, chunks[1]);

        // Status bar
        let status_text = if let Some(msg) = &self.state.status_message {
            msg.clone()
        } else {
            format!(
                "{} API: Connected | {} DB: Ready | [Enter] Details [Space] Multi-Select [Ctrl+F] Search [Q]uit [R]efresh",
                Emoji::API, Emoji::DATABASE
            )
        };

        let status = Paragraph::new(status_text)
            .style(self.theme.dim)
            .block(Block::default().borders(Borders::ALL).style(self.theme.border));
        frame.render_widget(status, chunks[2]);
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

        let search_box =
            Paragraph::new(format!("{} Search: {}", Emoji::SEARCH, self.state.search_query))
                .style(self.theme.title)
                .block(Block::default().borders(Borders::ALL).title("Device Search"));
        frame.render_widget(search_box, chunks[0]);

        // Filter and show matching devices
        let filtered: Vec<Device> = self
            .devices
            .iter()
            .filter(|d| {
                self.state.search_query.is_empty()
                    || d.name.to_lowercase().contains(&self.state.search_query.to_lowercase())
                    || d.model.to_lowercase().contains(&self.state.search_query.to_lowercase())
            })
            .cloned()
            .collect();

        let empty_selection = vec![];
        let list = widgets::device_list::render(&filtered, 0, &empty_selection, &self.theme);
        frame.render_widget(list, chunks[1]);
    }
}
