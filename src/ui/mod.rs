use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use tokio::time::{interval, Duration};

use crate::{api, config, db};

pub mod theme;
pub mod widgets;

use theme::Theme;

pub struct App {
    client: api::Client,
    db: db::Database,
    config: config::Config,
    theme: Theme,
    devices: Vec<api::Device>,
    selected_index: usize,
    should_quit: bool,
}

impl App {
    pub fn new(client: api::Client, db: db::Database, config: config::Config) -> Self {
        Self {
            client,
            db,
            config,
            theme: Theme::dark(),
            devices: Vec::new(),
            selected_index: 0,
            should_quit: false,
        }
    }

    pub async fn refresh_devices(&mut self) -> Result<()> {
        self.devices = self.client.get_devices().await?;

        // Save devices to database
        for device in &self.devices {
            self.db
                .save_device(&device.id, &device.name, &device.model)?;
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('r'), _) => {
                // Trigger refresh (handled in main loop)
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                if self.selected_index < self.devices.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new(format!(
            "{} Govee Controller - {} devices",
            theme::Emoji::HOME,
            self.devices.len()
        ))
        .style(self.theme.title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(self.theme.border),
        );

        frame.render_widget(title, chunks[0]);

        // Main area - device list
        let device_list =
            widgets::device_list::render(&self.devices, self.selected_index, &self.theme);
        frame.render_widget(device_list, chunks[1]);

        // Status bar
        let status = Paragraph::new(format!(
            "{} API: Connected | {} DB: Ready | [Q]uit [R]efresh",
            theme::Emoji::API,
            theme::Emoji::DATABASE
        ))
        .style(self.theme.dim)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(self.theme.border),
        );

        frame.render_widget(status, chunks[2]);
    }
}

pub async fn run(client: api::Client, db: db::Database, config: config::Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client, db, config);
    app.refresh_devices().await?;

    let mut refresh_interval = interval(Duration::from_secs(5));

    loop {
        terminal.draw(|f| app.render(f))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);
            }
        }

        // Auto-refresh (poll without blocking)
        tokio::select! {
            _ = refresh_interval.tick() => {
                let _ = app.refresh_devices().await;
            }
            else => {}
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
