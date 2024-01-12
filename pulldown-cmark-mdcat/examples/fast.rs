use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use pulldown_cmark_mdcat::bufferline::BufferLine;
use pulldown_cmark_mdcat::markdown_widget::FasterMarkdownWidget;
use pulldown_cmark_mdcat::markdown_widget::{Offset, PathOrStr};
use ratatui::prelude::*;
use std::{
    cell::OnceCell,
    io::{self, stdout},
    path::PathBuf,
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut offset = Offset { x: 0, y: 0 };
    let cell: OnceCell<Vec<_>> = OnceCell::new();

    'outer: loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let v = cell.get_or_init(|| {
                vec![PathOrStr::from_md_path(
                    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                        .parent()
                        .unwrap()
                        .join("sample")
                        .join("showcase.md"),
                )]
                .iter()
                .map(|x| {
                    let mut y = x.get_bufferlines(area);
                    y.push(BufferLine::Line(Vec::new()));
                    y
                })
                .flatten()
                .collect()
            });

            frame.render_stateful_widget(FasterMarkdownWidget { t: v }, area, &mut offset);
        })?;
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
