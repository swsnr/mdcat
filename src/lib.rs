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

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to a `writer`.
///
/// `push_tty` tries to limit output to the given number of TTY `columns` but
/// does not guarantee that output stays within the column limit.
pub fn push_tty<'a, 'e, W, I>(
    writer: &'a mut W,
    capabilities: &TerminalCapabilities,
    size: TerminalSize,
    mut events: I,
    base_dir: &'a Path,
    resource_access: ResourceAccess,
    syntax_set: SyntaxSet,
) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = Event<'e>>,
    W: Write,
{
    let theme = &ThemeSet::load_defaults().themes["Solarized (dark)"];
    events
        .try_fold(
            Context::new(
                writer,
                capabilities,
                size,
                base_dir,
                resource_access,
                syntax_set,
                theme,
            ),
            write_event,
        )?
        .write_pending_links()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use pulldown_cmark::Parser;

    fn render_string(
        input: &str,
        base_dir: &Path,
        resource_access: ResourceAccess,
        syntax_set: SyntaxSet,
        capabilities: TerminalCapabilities,
        size: TerminalSize,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let source = Parser::new(input);
        let mut sink = Vec::new();
        push_tty(
            &mut sink,
            &capabilities,
            size,
            source,
            base_dir,
            resource_access,
            syntax_set,
        )?;
        Ok(sink)
    }

    #[test]
    #[allow(non_snake_case)]
    fn GH_49_format_no_colour_simple() {
        let result = String::from_utf8(
            render_string(
                "_lorem_ **ipsum** dolor **sit** _amet_",
                Path::new("/"),
                ResourceAccess::LocalOnly,
                SyntaxSet::default(),
                TerminalCapabilities::none(),
                TerminalSize::default(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result, "lorem ipsum dolor sit amet\n");
    }
}
