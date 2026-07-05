use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::FutureExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::time::{interval, Duration};

use crate::{api, config};

pub mod app;
pub mod async_ops;
pub mod focus;
pub mod handlers;
pub mod layout;
pub mod renderer;
pub mod theme;
pub mod view_state;
pub mod widgets;

use app::App;

pub async fn run(client: api::Client, config: config::Config) -> Result<()> {
    // Restore the terminal before printing a panic, or the report is
    // unreadable and the shell is left in raw mode
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Device-list refreshes cost API budget (10,000 requests/day);
    // don't let a config typo turn this into a poll loop
    let refresh_ms = config.ui.refresh_interval_ms.max(10_000);
    let mut app = App::new(client, config);

    // The interval's first tick fires immediately, triggering the initial load
    let mut refresh_interval = interval(Duration::from_millis(refresh_ms));

    let result = run_loop(&mut terminal, &mut app, &mut refresh_interval).await;

    // Always restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
                if key.kind != KeyEventKind::Release {
                    app.handle_key(key.code, key.modifiers);

                    // Handle refresh request
                    if app.needs_refresh {
                        app.request_refresh_devices();
                        app.needs_refresh = false;
                    }
                }
            }
        }

        // Expire status messages, flush debounced controls
        app.tick();

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
