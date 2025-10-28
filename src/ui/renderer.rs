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
                _ => {}
            }
            return;
        }

        // Always render the multi-pane layout
        let layout = MultiPaneLayout::new(frame);

        // Render overview panel (top)
        widgets::overview::render(&self.devices, self.loading, &self.theme, frame, layout.overview);

        // Render device list (left) with focus indicator
        let list_style = if self.state.focus == Focus::List {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        let device_list = widgets::device_list::render_with_style(
            &self.devices,
            self.state.selected_index,
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
        widgets::footer::render(
            &self.state.focus,
            &self.theme,
            frame,
            layout.footer,
        );
    }

    fn render_help_modal(&self, frame: &mut Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout, Rect},
            widgets::{Block, Borders, Clear, Paragraph},
        };
        use crate::ui::theme::Emoji;

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
            "  q           Quit application".to_string(),
            "  ?           Show/hide this help".to_string(),
            "  r           Refresh devices".to_string(),
            "  Tab         Switch focus (List ↔ Detail)".to_string(),
            "".to_string(),
            "═══ DEVICE LIST (when focused) ═══".to_string(),
            "  ↑/↓, j/k    Navigate devices".to_string(),
            "  Enter       Focus detail pane".to_string(),
            "".to_string(),
            "═══ DEVICE DETAIL (when focused) ===".to_string(),
            "  Space       Toggle power ON/OFF".to_string(),
            "  ↑/↓, j/k    Adjust brightness ±10%".to_string(),
            "  Shift+↑/↓   Adjust brightness ±5% (fine control)".to_string(),
            "  c           Open color picker".to_string(),
            "  Esc         Back to list focus".to_string(),
            "".to_string(),
            "═══ COLOR PICKER ═══".to_string(),
            "  Tab         Switch R/G/B channel".to_string(),
            "  ↑/↓         Adjust value".to_string(),
            "  Enter       Apply color".to_string(),
            "  Esc         Cancel".to_string(),
            "".to_string(),
            "═══ VISUAL CUES ═══".to_string(),
            "  Blue border   = Focused pane".to_string(),
            "  📦            = Device group".to_string(),
            "  💡            = Individual device".to_string(),
            "  ✅ ON         = Power on".to_string(),
            "  ⭕ OFF        = Power off".to_string(),
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
}
