use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Htop-style multi-pane layout with fixed regions
pub struct MultiPaneLayout {
    pub overview: Rect,       // Top: Stats and overview
    pub device_list: Rect,    // Middle-left: Device list
    pub device_detail: Rect,  // Middle-right: Device detail
    pub status: Rect,         // Bottom-middle: Status/error messages
    pub footer: Rect,         // Bottom: Key bindings
}

impl MultiPaneLayout {
    pub fn new(frame: &Frame) -> Self {
        let area = frame.area();

        // Vertical split: Overview | Main | Status | Footer
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Overview panel
                Constraint::Min(10),    // Main content area (device list + detail)
                Constraint::Length(4),  // Status/error panel
                Constraint::Length(3),  // Footer with key bindings
            ])
            .split(area);

        // Split main area horizontally: Device List | Device Detail
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // Device list
                Constraint::Percentage(60),  // Device detail
            ])
            .split(vertical_chunks[1]);

        Self {
            overview: vertical_chunks[0],
            device_list: horizontal_chunks[0],
            device_detail: horizontal_chunks[1],
            status: vertical_chunks[2],
            footer: vertical_chunks[3],
        }
    }
}
