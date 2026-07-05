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
                    // Render the main layout behind the popup for context
                    self.render_main(frame);
                    widgets::color_picker::render(picker, &self.theme, frame);
                }
                Modal::ScenePicker(picker) => {
                    // Render the main layout behind the popup for context
                    self.render_main(frame);
                    widgets::scene_picker::render(picker, &self.theme, frame);
                }
                _ => {}
            }
            return;
        }

        self.render_main(frame);
    }

    fn render_main(&self, frame: &mut Frame) {
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
        widgets::device_list::render_with_style(
            &self.devices,
            &self.known_states,
            self.state.selected_index,
            self.loading,
            &self.theme,
            frame,
            layout.device_list,
            list_style,
        );

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
                self.state_loading,
                &self.theme,
                frame,
                layout.device_detail,
                detail_style,
            );
        } else {
            use ratatui::widgets::{Block, Borders, Paragraph};
            let placeholder = if self.loading {
                "Loading…"
            } else {
                "No device selected"
            };
            let empty = Paragraph::new(placeholder).style(self.theme.dim).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(detail_style),
            );
            frame.render_widget(empty, layout.device_detail);
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
            layout::{Constraint, Direction, Layout, Rect},
            widgets::{Block, Borders, Clear, Paragraph},
        };

        // Two columns of ~24 lines each; size the popup to fit
        let area = frame.area();
        let width = area.width.min(84);
        let height = area.height.min(28);
        let popup_area = Rect {
            x: area.width.saturating_sub(width) / 2,
            y: area.height.saturating_sub(height) / 2,
            width,
            height,
        };

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!(" {} Help ", Emoji::HELP))
            .title_bottom(" press any key to close ")
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let left = [
            "═ GLOBAL ═",
            " q          Quit",
            " ?          This help",
            " r          Refresh devices",
            " Tab        Switch focus",
            "",
            "═ DEVICE LIST ═",
            " ↑/↓, j/k   Navigate",
            " Space      Toggle power",
            " Enter      Detail pane",
            "",
            "═ DEVICE DETAIL ═",
            " Space      Toggle power",
            " ↑/↓, j/k   Brightness ±10%",
            " Shift+↑/↓  Brightness ±5%",
            " ←/→, h/l   Temp ±500K",
            " Shift+←/→  Temp ±100K",
            " c          Color picker",
            " s          Scene picker",
            " Esc        Back to list",
        ];
        let right = [
            "═ COLOR PICKER (RGB) ═",
            " ↑/↓        R/G/B channel",
            " ←/→        Adjust ±10",
            " Tab        Color browser",
            " Enter      Apply",
            "",
            "═ COLOR PICKER (browser) ═",
            " ↑/↓        Browse colors",
            " ←/→        Switch group",
            " Tab        RGB editor",
            " Enter      Apply",
            "",
            "═ SCENE PICKER ═",
            " ↑/↓, j/k   Browse scenes",
            " Enter      Apply scene",
            "",
            "═ VISUAL CUES ═",
            " cyan border  focused pane",
            " 📦 group   💡 device",
            " ✅ on  ⭕ off  · unknown",
        ];

        frame.render_widget(
            Paragraph::new(left.join("\n")).style(self.theme.text),
            columns[0],
        );
        frame.render_widget(
            Paragraph::new(right.join("\n")).style(self.theme.text),
            columns[1],
        );
    }
}
