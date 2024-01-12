use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use pulldown_cmark_mdcat::markdown_widget::{MarkdownWidget, Offset, PathOrStr};
use ratatui::prelude::*;
use std::{
    io::{self, stdout},
    path::PathBuf,
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut offset = Offset { x: 0, y: 0 };

    'outer: loop {
        terminal.draw(|f| ui(f, &mut offset))?;
        loop {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match &key.code {
                        KeyCode::Char('q') => {
                            break 'outer;
                        }
                        KeyCode::Char('j') => {
                            offset.y = offset.y.saturating_add(1);
                            break;
                        }
                        KeyCode::Char('k') => {
                            offset.y = offset.y.saturating_sub(1);
                            break;
                        }
                        KeyCode::Home => {
                            offset.y = 0;
                            break;
                        }
                        KeyCode::End => {
                            offset.y = u16::MAX;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(frame: &mut Frame, offset: &mut Offset) {
    let layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(frame.size());
    frame.render_stateful_widget(
        MarkdownWidget {
            path_or_str: &vec![PathOrStr::from_md_path(
                PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                    .parent()
                    .unwrap()
                    .join("sample")
                    .join("showcase.md"),
            )],
        },
        layout[1],
        offset,
    );
}
