use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::FutureExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::time::{interval, Duration};

use crate::{api, config, db};

pub mod app;
pub mod async_ops;
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

    // Request initial device load (non-blocking)
    app.request_refresh_devices();

    let mut refresh_interval = interval(Duration::from_secs(30));

    // Ensure terminal is restored even on panic or Ctrl-C
    let result = run_loop(&mut terminal, &mut app, &mut refresh_interval).await;

    // Always restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    refresh_interval: &mut tokio::time::Interval,
) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        // Poll for async responses (non-blocking)
        while let Ok(response) = app.resp_rx.try_recv() {
            app.handle_async_response(response);
        }

        // Handle events with timeout
        if event::poll(Duration::from_millis(16))? {
            // 60 FPS
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);

                // Handle async actions (non-blocking requests)
                match &app.state.view_mode {
                    ViewMode::Detail => {
                        // Load state if entering detail view
                        if app.state.device_state.is_none() {
                            app.request_load_device_state();
                        }
                    }
                    ViewMode::Brightness => {
                        // Apply brightness on Enter
                        if matches!(key.code, crossterm::event::KeyCode::Enter) {
                            if let Some(brightness) = &app.state.brightness_control {
                                let value = brightness.value;
                                app.request_apply_brightness(value);
                                app.state.exit_to_detail();
                            }
                        }
                    }
                    ViewMode::ColorPicker => {
                        // Apply color on Enter
                        if matches!(key.code, crossterm::event::KeyCode::Enter) {
                            if let Some(picker) = &app.state.color_picker {
                                let (r, g, b) = (picker.r, picker.g, picker.b);
                                app.request_apply_color(r, g, b);
                                app.state.exit_to_detail();
                            }
                        }
                    }
                    _ => {}
                }

                // Handle refresh request
                if app.needs_refresh {
                    app.request_refresh_devices();
                    app.needs_refresh = false;
                }
            }
        }

        // Auto-refresh (non-blocking)
        if refresh_interval.tick().now_or_never().is_some() {
            app.request_refresh_devices();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
