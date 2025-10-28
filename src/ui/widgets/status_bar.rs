use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::Theme;

pub fn render(
    status_message: Option<&String>,
    error_message: Option<&String>,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
) {
    let lines = if let Some(err) = error_message {
        vec![
            Line::from(Span::styled("⚠️  ERROR", theme.error)),
            Line::from(Span::styled(err.clone(), theme.error)),
        ]
    } else if let Some(msg) = status_message {
        vec![Line::from(Span::styled(msg.clone(), theme.success))]
    } else {
        vec![Line::from(Span::styled("Ready", theme.dim))]
    };

    let status = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .style(theme.border),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(status, area);
}
