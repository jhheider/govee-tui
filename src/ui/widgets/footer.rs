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
            "[↑↓/jk] navigate  [space] power  [enter] detail  [tab] switch  [r] refresh  [?] help  [q] quit"
        }
        Focus::Detail => {
            "[space] power  [↑↓/kj] brightness  [←→/hl] temp  [c] color  [s] scenes  [esc] list  [?] help"
        }
    };

    let footer = Paragraph::new(Line::from(Span::styled(keybindings, theme.text)))
        .block(Block::default().borders(Borders::ALL).style(theme.border));

    frame.render_widget(footer, area);
}
