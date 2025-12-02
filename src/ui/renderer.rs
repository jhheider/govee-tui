use ratatui::Frame;

use super::app::App;
use super::layout::MultiPaneLayout;
use super::view_state::Modal;
use super::widgets;
use crate::ui::focus::Focus;

impl App {
    pub fn render(&self, frame: &mut Frame) {
        // Check if we should render a modal overlay
        if !matches!(self.state.modal, Modal::None) {
            match &self.state.modal {
                Modal::Help => self.render_help_modal(frame),
                Modal::ColorPicker(picker) => {
                    widgets::color_picker::render(picker, &self.theme, frame);
                }
                Modal::Search => self.render_search_modal(frame),
                Modal::Scenes => self.render_scenes_modal(frame),
                Modal::None => {}
            }
            return;
        }

        // Always render the multi-pane layout
        let layout = MultiPaneLayout::new(frame);

        // Render overview panel (top)
        widgets::overview::render(
            &self.devices,
            self.loading,
            &self.theme,
            frame,
            layout.overview,
        );

        // Render device list (left) with focus indicator
        let list_style = if self.state.focus == Focus::List {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        let device_list = widgets::device_list::render_with_style(
            &self.devices,
            self.state.selected_index,
            &self.state.selected_devices,
            &self.state.focus,
            &self.state.search_query,
            &self.theme,
            list_style,
        );
        frame.render_widget(device_list, layout.device_list);

        // Render device detail (right) with focus indicator
        let detail_style = if self.state.focus == Focus::Detail {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        if let Some(device) = self.selected_device() {
            widgets::detail_view::render_with_style(
                device,
                self.state.device_state.as_ref(),
                self.loading,
                &self.theme,
                frame,
                layout.device_detail,
                detail_style,
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
        widgets::footer::render(&self.state.focus, &self.theme, frame, layout.footer);
    }

    fn render_help_modal(&self, frame: &mut Frame) {
        use crate::ui::theme::Emoji;
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Clear, Paragraph},
        };

        // Center the modal
        let area = frame.area();
        let popup_area = Rect {
            x: area.width / 6,
            y: area.height / 10,
            width: area.width * 2 / 3,
            height: area.height * 4 / 5,
        };

        // Clear the area
        frame.render_widget(Clear, popup_area);

        let help_text = vec![
            format!("{} GOVEE TUI - KEYBOARD SHORTCUTS", Emoji::HELP),
            "".to_string(),
            "═══ GLOBAL ═══".to_string(),
            "  q, Ctrl+C   Quit application".to_string(),
            "  ?           Show/hide this help".to_string(),
            "  r           Refresh devices".to_string(),
            "  /           Search/filter devices".to_string(),
            "  Tab         Switch focus (List ↔ Detail)".to_string(),
            "".to_string(),
            "═══ DEVICE LIST ═══".to_string(),
            "  ↑/↓, j/k    Navigate devices".to_string(),
            "  g/G         Jump to first/last device".to_string(),
            "  Space       Quick power toggle".to_string(),
            "  Enter       Focus detail pane".to_string(),
            "  x           Toggle selection (multi-device)".to_string(),
            "  a           Select all  |  A  Deselect all".to_string(),
            "".to_string(),
            "═══ DEVICE DETAIL ═══".to_string(),
            "  Space       Toggle power ON/OFF".to_string(),
            "  ↑/↓, j/k    Adjust brightness ±10%".to_string(),
            "  J/K         Adjust brightness ±5% (fine)".to_string(),
            "  1-9, 0      Set brightness 10%-100%".to_string(),
            "  c           Open color picker".to_string(),
            "  t/T         Adjust temp ±500K (warm/cool)".to_string(),
            "  s           Open scenes browser".to_string(),
            "  Esc         Back to list".to_string(),
            "".to_string(),
            "═══ COLOR PICKER ═══".to_string(),
            "  Tab         Switch mode (RGB ↔ Browser)".to_string(),
            "  ↑/↓         Select channel / color".to_string(),
            "  ←/→         Adjust value / group".to_string(),
            "  Enter       Apply color".to_string(),
            "  Esc         Cancel".to_string(),
            "".to_string(),
            "Press any key to close...".to_string(),
        ];

        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .style(self.theme.text)
            .block(
                Block::default()
                    .title(format!(" {} Help ", Emoji::HELP))
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_focused),
            );

        frame.render_widget(help_paragraph, popup_area);
    }

    fn render_search_modal(&self, frame: &mut Frame) {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Clear, Paragraph},
        };

        let area = frame.area();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: 5,
        };

        frame.render_widget(Clear, popup_area);

        let search_text = format!(
            "🔍 Search: {}█\n\nType to filter devices. [Enter] Apply  [Esc] Cancel",
            self.state.search_query
        );

        let search_widget = Paragraph::new(search_text)
            .style(self.theme.text)
            .block(
                Block::default()
                    .title(" Search Devices ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_focused),
            );

        frame.render_widget(search_widget, popup_area);
    }

    fn render_scenes_modal(&self, frame: &mut Frame) {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Clear, Paragraph},
        };

        let area = frame.area();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        frame.render_widget(Clear, popup_area);

        // TODO: Load actual scenes from API
        let scenes_text = [
            "🎭 Scenes",
            "",
            "  Coming soon!",
            "",
            "  Scenes will be loaded from your",
            "  Govee account and displayed here.",
            "",
            "[Esc] Close",
        ];

        let scenes_widget = Paragraph::new(scenes_text.join("\n"))
            .style(self.theme.text)
            .block(
                Block::default()
                    .title(" Dynamic Scenes ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_focused),
            );

        frame.render_widget(scenes_widget, popup_area);
    }
}
