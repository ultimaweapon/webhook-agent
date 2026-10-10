use crossterm::event::{Event, EventStream, KeyCode};
use erdp::ErrorDisplay;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::prelude::{Buffer, Rect, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};
use std::process::ExitCode;
use thiserror::Error;
use tokio::select;
use tokio_util::sync::CancellationToken;

fn main() -> ExitCode {
    // Build async runtime.
    let tokio = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build_local(Default::default())
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to build Tokio runtime: {}.", e.display());
            return ExitCode::FAILURE;
        }
    };

    // Initialize application.
    let term = ratatui::init();
    let app = App {
        redraw: true,
        running: CancellationToken::new(),
    };

    // Run.
    let exit = tokio.block_on(app.run(term));

    ratatui::restore();

    match exit {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Failed to run application: {}.", e.display());
            ExitCode::FAILURE
        }
    }
}

/// Global states for program.
struct App {
    redraw: bool,
    running: CancellationToken,
}

impl App {
    async fn run(mut self, mut term: DefaultTerminal) -> Result<(), AppError> {
        // Dispatch event til exit.
        let mut events = EventStream::new();

        loop {
            if std::mem::take(&mut self.redraw)
                && let Err(e) = term.draw(|f| f.render_widget(&self, f.area()))
            {
                return Err(AppError::Draw(e));
            }

            select! {
                biased;
                _ = self.running.cancelled() => break Ok(()),
                v = events.next() => match v {
                    Some(Ok(e)) => self.dispatch_event(e),
                    Some(Err(e)) => break Err(AppError::WaitForEvent(e)),
                    None => unreachable!(),
                },
            }
        }
    }

    fn dispatch_event(&mut self, e: Event) {
        match e {
            Event::Key(e) => match e.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => self.running.cancel(),
                _ => (),
            },
            Event::Resize(_, _) => self.redraw = true,
            _ => (),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" Webhook Agent ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        block.render(area, buf);
    }
}

/// Reason why [App] fails no run.
#[derive(Debug, Error)]
enum AppError {
    #[error("couldn't wait for terminal event")]
    WaitForEvent(#[source] std::io::Error),

    #[error("couldn't draw terminal")]
    Draw(#[source] std::io::Error),
}
