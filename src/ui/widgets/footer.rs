use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;
use crate::ui::view_state::ViewMode;

pub fn render(view_mode: &ViewMode, has_selection: bool, theme: &Theme, frame: &mut Frame, area: Rect) {
    let keybindings = match view_mode {
        ViewMode::List => {
            if has_selection {
                "[↑↓/jk] Navigate  [Enter] Details  [Space] Toggle  [R] Refresh  [?] Help  [Q] Quit"
            } else {
                "[↑↓/jk] Navigate  [Enter] Details  [Space] Toggle  [R] Refresh  [?] Help  [Q] Quit"
            }
        }
        ViewMode::Detail => {
            "[Space] Power  [↑↓] Brightness  [C] Color  [Esc] Back  [?] Help"
        }
        ViewMode::ColorPicker => {
            "[Tab] Channel  [↑↓] Adjust  [Enter] Apply  [Esc] Cancel"
        }
        ViewMode::Brightness => {
            "[↑↓] Adjust  [1-9] Set %  [Enter] Apply  [Esc] Cancel"
        }
        ViewMode::Search => {
            "[Type] Filter  [Backspace] Delete  [Enter] Done  [Esc] Cancel"
        }
        ViewMode::Help => {
            "[Any Key] Close Help"
        }
    };

    let footer = Paragraph::new(Line::from(Span::styled(keybindings, theme.text)))
        .block(Block::default().borders(Borders::ALL).style(theme.border));

    frame.render_widget(footer, area);
}
