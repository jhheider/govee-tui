use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::focus::Focus;
use crate::ui::theme::Theme;

pub fn render(focus: &Focus, theme: &Theme, frame: &mut Frame, area: Rect) {
    // Compact hints on narrow terminals so nothing truncates mid-word
    let narrow = area.width < 96;
    let keybindings = match (focus, narrow) {
        (Focus::List, false) => {
            "[↑↓/jk] navigate  [space] power  [enter] detail  [tab] switch  [r] refresh  [?] help  [q] quit"
        }
        (Focus::List, true) => "[↑↓/jk] move [space] power [enter] detail [?] help [q] quit",
        (Focus::Detail, false) => {
            "[space] power  [↑↓/jk] brightness  [←→/hl] temp  [c] color  [s] scenes  [esc] list  [?] help"
        }
        (Focus::Detail, true) => "[space] power [↑↓] bright [←→] temp [c] color [s] scenes [esc] back",
    };

    let footer = Paragraph::new(Line::from(Span::styled(keybindings, theme.text)))
        .block(Block::default().borders(Borders::ALL).style(theme.border));

    frame.render_widget(footer, area);
}
