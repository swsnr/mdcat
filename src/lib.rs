// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![deny(warnings, missing_docs, clippy::all)]

//! Write markdown to TTYs.

use pulldown_cmark::Event;
use std::error::Error;
use std::io::Write;
use std::path::Path;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

mod magic;
mod resources;
mod svg;
mod terminal;

mod context_write;

use context_write::*;

// Expose some select things for use in main
pub use crate::resources::ResourceAccess;
pub use crate::terminal::*;

/// Dump markdown events to a writer.
pub fn dump_events<'a, W, I>(writer: &mut W, events: I) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    for event in events {
        writeln!(writer, "{:?}", event)?;
    }
    Ok(())
}

/// Settings for markdown rendering.
#[derive(Debug)]
pub struct Settings {
    /// Capabilities of the terminal mdcat writes to.
    pub terminal_capabilities: TerminalCapabilities,
    /// The size of the terminal mdcat writes to.
    pub terminal_size: TerminalSize,
    /// Whether remote resource access is permitted.
    pub resource_access: ResourceAccess,
    /// Syntax set for syntax highlighting of code blocks.
    pub syntax_set: SyntaxSet,
}

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to a `writer`, using the given `settings` for rendering and
/// resource access.  `base_dir` denotes the base directory the `events` were
/// read from, to resolve relative references in the Markdown document.
///
/// `push_tty` tries to limit output to the given number of TTY `columns` but
/// does not guarantee that output stays within the column limit.
pub fn push_tty<'a, 'e, W, I>(
    settings: &Settings,
    writer: &'a mut W,
    base_dir: &'a Path,
    mut events: I,
) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = Event<'e>>,
    W: Write,
{
    let theme = &ThemeSet::load_defaults().themes["Solarized (dark)"];
    events
        .try_fold(Context::new(writer, settings, base_dir, theme), write_event)?
        .write_pending_links()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use pulldown_cmark::Parser;

    fn render_string(input: &str, settings: &Settings) -> Result<Vec<u8>, Box<dyn Error>> {
        let source = Parser::new(input);
        let mut sink = Vec::new();
        push_tty(settings, &mut sink, &Path::new("/"), source)?;
        Ok(sink)
    }

    #[test]
    #[allow(non_snake_case)]
    fn GH_49_format_no_colour_simple() {
        let result = String::from_utf8(
            render_string(
                "_lorem_ **ipsum** dolor **sit** _amet_",
                &Settings {
                    resource_access: ResourceAccess::LocalOnly,
                    syntax_set: SyntaxSet::default(),
                    terminal_capabilities: TerminalCapabilities::none(),
                    terminal_size: TerminalSize::default(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result, "lorem ipsum dolor sit amet\n");
    }
}
