use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::Device;
use crate::ui::theme::{Emoji, Theme};

pub fn render(devices: &[Device], loading: bool, theme: &Theme, frame: &mut Frame, area: Rect) {
    let (total, groups, individuals) = count_devices(devices);
    let controllable = devices.iter().filter(|d| d.controllable).count();

    let status = if loading {
        format!(" {} Loading…", Emoji::LOADING)
    } else {
        "".to_string()
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("{} Govee Controller", Emoji::HOME), theme.title),
            Span::styled(status, theme.dim),
        ]),
        Line::from(vec![
            Span::raw(format!("Total: {total} ")),
            Span::styled(
                format!("({groups} 📦 groups, {individuals} 💡 devices)"),
                theme.dim,
            ),
            Span::raw("  "),
            Span::styled(format!("Controllable: {controllable}"), theme.text),
        ]),
    ];

    let overview =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).style(theme.border));

    frame.render_widget(overview, area);
}

fn count_devices(devices: &[Device]) -> (usize, usize, usize) {
    let total = devices.len();
    let groups = devices.iter().filter(|d| d.is_group).count();
    let individuals = total - groups;
    (total, groups, individuals)
}
