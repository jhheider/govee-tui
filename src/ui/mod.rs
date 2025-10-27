use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::time::{interval, Duration};

use crate::{api, config, db};

pub mod app;
pub mod handlers;
pub mod renderer;
pub mod theme;
pub mod view_state;
pub mod widgets;

use app::App;
use view_state::ViewMode;

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

                // Handle async actions
                match &app.state.view_mode {
                    ViewMode::Detail => {
                        // Load state if entering detail view
                        if app.state.device_state.is_none() {
                            let _ = app.load_device_state().await;
                        }
                    }
                    ViewMode::Brightness => {
                        // Apply brightness on Enter
                        if matches!(key.code, crossterm::event::KeyCode::Enter) {
                            if let Some(brightness) = &app.state.brightness_control {
                                let value = brightness.value;
                                let _ = app.apply_brightness(value).await;
                                app.state.exit_to_detail();
                            }
                        }
                    }
                    ViewMode::ColorPicker => {
                        // Apply color on Enter
                        if matches!(key.code, crossterm::event::KeyCode::Enter) {
                            if let Some(picker) = &app.state.color_picker {
                                let (r, g, b) = (picker.r, picker.g, picker.b);
                                let _ = app.apply_color(r, g, b).await;
                                app.state.exit_to_detail();
                            }
                        }
                    }
                    _ => {}
                }

                // Handle refresh request
                if app.needs_refresh {
                    let _ = app.refresh_devices().await;
                    app.needs_refresh = false;
                }
            }
        }

        // Auto-refresh
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
