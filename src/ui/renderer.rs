use ratatui::Frame;

use super::app::App;
use super::layout::MultiPaneLayout;
use super::view_state::ViewMode;
use super::widgets;

impl App {
    pub fn render(&self, frame: &mut Frame) {
        // Always render the multi-pane layout
        let layout = MultiPaneLayout::new(frame);

        // Render overview panel (top)
        widgets::overview::render(&self.devices, self.loading, &self.theme, frame, layout.overview);

        // Render device list (left)
        let device_list = widgets::device_list::render(
            &self.devices,
            self.state.selected_index,
            &self.state.selected_devices,
            &self.theme,
        );
        frame.render_widget(device_list, layout.device_list);

        // Render device detail (right)
        if let Some(device) = self.selected_device() {
            widgets::detail_view::render(
                device,
                self.state.device_state.as_ref(),
                &self.theme,
                frame,
                layout.device_detail,
            );
        }

        // Render status/error panel (bottom-middle)
        widgets::status_bar::render(
            self.state.status_message.as_ref(),
            self.state.error_message.as_ref(),
            &self.theme,
            frame,
            layout.status,
        );

        // Render footer with context-dependent keybindings (bottom)
        widgets::footer::render(
            &self.state.view_mode,
            !self.state.selected_devices.is_empty(),
            &self.theme,
            frame,
            layout.footer,
        );

        // Render overlays on top if needed
        match &self.state.view_mode {
            ViewMode::Brightness => self.render_brightness_overlay(frame),
            ViewMode::ColorPicker => self.render_color_picker_overlay(frame),
            ViewMode::Search => self.render_search_overlay(frame),
            ViewMode::Help => self.render_help_overlay(frame),
            _ => {}
        }
    }


    fn render_brightness_overlay(&self, frame: &mut Frame) {
        if let Some(brightness) = &self.state.brightness_control {
            widgets::brightness::render(brightness, &self.theme, frame);
        }
    }

    fn render_color_picker_overlay(&self, frame: &mut Frame) {
        if let Some(picker) = &self.state.color_picker {
            widgets::color_picker::render(picker, &self.theme, frame);
        }
    }

    fn render_search_overlay(&self, frame: &mut Frame) {
        // Centered overlay for search
        use ratatui::{
            layout::{Constraint, Direction, Layout, Rect},
            text::Line,
            widgets::{Block, Borders, Paragraph},
        };
        use crate::ui::theme::Emoji;

        let area = frame.area();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: 8,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(popup_area);

        let search_box = Paragraph::new(format!("{} Search: {}", Emoji::SEARCH, self.state.search_query))
            .style(self.theme.title)
            .block(Block::default().borders(Borders::ALL).title("Device Search"));
        frame.render_widget(search_box, chunks[0]);

        // Show filtered devices
        let query_lower = self.state.search_query.to_lowercase();
        let count = self
            .devices
            .iter()
            .filter(|d| {
                query_lower.is_empty()
                    || d.name.to_lowercase().contains(&query_lower)
                    || d.model.to_lowercase().contains(&query_lower)
            })
            .count();

        let result_text = format!("Found {} matching device(s)", count);
        let result = Paragraph::new(Line::from(result_text))
            .style(self.theme.text)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(result, chunks[1]);
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use crate::ui::theme::Emoji;

        let help_text = vec![
            format!("{} Govee TUI - Keyboard Shortcuts", Emoji::HELP),
            "".to_string(),
            "═══ GENERAL ═══".to_string(),
            "  q           Quit".to_string(),
            "  ?           Show/hide this help".to_string(),
            "  r           Refresh devices".to_string(),
            "".to_string(),
            "═══ NAVIGATION ═══".to_string(),
            "  ↑/↓, j/k    Navigate devices".to_string(),
            "  Enter       Focus device detail".to_string(),
            "  Esc         Back to list".to_string(),
            "".to_string(),
            "═══ DEVICE CONTROL ═══".to_string(),
            "  Space       Toggle power ON/OFF".to_string(),
            "  ↑/↓         Brightness ±10%".to_string(),
            "  Shift+↑/↓   Brightness ±5%".to_string(),
            "  c           Change color".to_string(),
            "".to_string(),
            "═══ MULTI-SELECT ═══".to_string(),
            "  Space       Toggle selection (in list)".to_string(),
            "  x           Clear all selections".to_string(),
            "".to_string(),
            "Press any key to close help...".to_string(),
        ];

        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .style(self.theme.text)
            .block(
                Block::default()
                    .title(format!("{} Help", Emoji::HELP))
                    .borders(Borders::ALL)
                    .style(self.theme.border),
            );

        frame.render_widget(help_paragraph, frame.area());
    }
}
