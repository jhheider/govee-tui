use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::focus::Focus;
use crate::ui::theme::Theme;

pub fn render(focus: &Focus, theme: &Theme, frame: &mut Frame, area: Rect) {
    let keybindings = match focus {
        Focus::List => {
            "[↑↓/jk] Navigate  [Space] Power  [Enter] Details  [/] Search  [?] Help  [Q] Quit"
        }
        Focus::Detail => {
            "[Space] Power  [↑↓] Bright  [C] Color  [t/T] Temp  [1-9] Quick%  [Esc] Back  [?] Help"
        }
    };

    let footer = Paragraph::new(Line::from(Span::styled(keybindings, theme.text)))
        .block(Block::default().borders(Borders::ALL).style(theme.border));

    frame.render_widget(footer, area);
}
