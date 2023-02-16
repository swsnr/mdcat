// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, missing_docs, clippy::all)]

//! Write markdown to TTYs.

use std::io::{ErrorKind, Result, Write};
use std::path::Path;

use pulldown_cmark::Event;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tracing::instrument;

// Expose some select things for use in main
#[cfg(feature = "render-image")]
pub use crate::resources::ResourceAccess;
use crate::terminal::capabilities::TerminalCapabilities;
use crate::terminal::TerminalSize;
use url::Url;

#[cfg(feature = "render-image")]
mod magic;
mod references;
#[cfg(feature = "render-image")]
mod resources;
#[cfg(feature = "render-image")]
mod svg;
pub mod terminal;

mod render;

/// The mdcat error type.
///
/// This is `std::io::Error`: mdcat never fails visible unless it cannot write output.
pub type Error = std::io::Error;

/// Settings for markdown rendering.
#[derive(Debug)]
pub struct Settings {
    /// Capabilities of the terminal mdcat writes to.
    pub terminal_capabilities: TerminalCapabilities,
    /// The size of the terminal mdcat writes to.
    pub terminal_size: TerminalSize,
    /// Whether remote resource access is permitted.
    #[cfg(feature = "render-image")]
    pub resource_access: ResourceAccess,
    /// Syntax set for syntax highlighting of code blocks.
    pub syntax_set: SyntaxSet,
}

/// The environment to render markdown in.
#[derive(Debug)]
pub struct Environment {
    /// The base URL to resolve relative URLs with.
    pub base_url: Url,
    /// The local host name.
    pub hostname: String,
}

impl Environment {
    /// Create an environment for the local host with the given `base_url`.
    ///
    /// Take the local hostname from `gethostname`.
    pub fn for_localhost(base_url: Url) -> Result<Self> {
        gethostname::gethostname()
            .into_string()
            .map_err(|raw| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("gethostname() returned invalid unicode data: {raw:?}"),
                )
            })
            .map(|hostname| Environment { base_url, hostname })
    }

    /// Create an environment for a local diretory.
    ///
    /// Convert the directory to a directory URL, and obtain the hostname from `gethostname`.
    ///
    /// `base_dir` must be an absolute path; return an IO error with `ErrorKind::InvalidInput`
    /// otherwise.
    pub fn for_local_directory<P: AsRef<Path>>(base_dir: &P) -> Result<Self> {
        Url::from_directory_path(base_dir)
            .map_err(|_| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Base directory {} must be an absolute path",
                        base_dir.as_ref().display()
                    ),
                )
            })
            .and_then(Self::for_localhost)
    }
}

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to a `writer`, using the given `settings` and `environment`
/// for rendering and resource access.
///
/// `push_tty` tries to limit output to the given number of TTY `columns` but
/// does not guarantee that output stays within the column limit.
#[instrument(level = "debug", skip_all, fields(environment.hostname = environment.hostname.as_str(), environment.base_url = &environment.base_url.as_str()))]
pub fn push_tty<'a, 'e, W, I>(
    settings: &Settings,
    environment: &Environment,
    writer: &'a mut W,
    mut events: I,
) -> Result<()>
where
    I: Iterator<Item = Event<'e>>,
    W: Write,
{
    let theme = &ThemeSet::load_defaults().themes["Solarized (dark)"];
    use render::*;
    let StateAndData(final_state, final_data) = events.try_fold(
        StateAndData(State::default(), StateData::default()),
        |StateAndData(state, data), event| {
            write_event(writer, settings, environment, theme, state, data, event)
        },
    )?;
    finish(writer, settings, environment, final_state, final_data)
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::Parser;

    use super::*;

    fn render_string(input: &str, settings: &Settings) -> anyhow::Result<String> {
        let source = Parser::new(input);
        let mut sink = Vec::new();
        let env =
            Environment::for_local_directory(&std::env::current_dir().expect("Working directory"))?;
        push_tty(settings, &env, &mut sink, source)?;
        Ok(String::from_utf8_lossy(&sink).into())
    }

    mod layout {
        use anyhow::Result;
        use pretty_assertions::assert_eq;
        use syntect::parsing::SyntaxSet;

        use crate::terminal::TerminalProgram;
        use crate::*;

        use super::render_string;

        fn render(markup: &str) -> Result<String> {
            render_string(
                markup,
                &Settings {
                    #[cfg(feature = "render-image")]
                    resource_access: ResourceAccess::LocalOnly,
                    syntax_set: SyntaxSet::default(),
                    terminal_capabilities: TerminalProgram::Dumb.capabilities(),
                    terminal_size: TerminalSize::default(),
                },
            )
        }

        #[test]
        #[allow(non_snake_case)]
        fn GH_49_format_no_colour_simple() {
            assert_eq!(
                render("_lorem_ **ipsum** dolor **sit** _amet_").unwrap(),
                "lorem ipsum dolor sit amet\n",
            )
        }

        #[test]
        fn begins_with_rule() {
            assert_eq!(render("----").unwrap(), "════════════════════════════════════════════════════════════════════════════════\n")
        }

        #[test]
        fn begins_with_block_quote() {
            assert_eq!(render("> Hello World").unwrap(), "    Hello World\n")
        }

        #[test]
        fn rule_in_block_quote() {
            assert_eq!(
                render(
                    "> Hello World

> ----"
                )
                .unwrap(),
                "    Hello World

    ════════════════════════════════════════════════════════════════════════════\n"
            )
        }

        #[test]
        fn heading_in_block_quote() {
            assert_eq!(
                render(
                    "> Hello World

> # Hello World"
                )
                .unwrap(),
                "    Hello World

    ┄Hello World\n"
            )
        }

        #[test]
        fn heading_levels() {
            assert_eq!(
                render(
                    "
# First

## Second

### Third"
                )
                .unwrap(),
                "┄First

┄┄Second

┄┄┄Third\n"
            )
        }

        #[test]
        fn autolink_creates_no_reference() {
            assert_eq!(
                render("Hello <http://example.com>").unwrap(),
                "Hello http://example.com\n"
            )
        }

        #[test]
        fn flush_ref_links_before_toplevel_heading() {
            assert_eq!(
                render(
                    "> Hello [World](http://example.com/world)

> # No refs before this headline

# But before this"
                )
                .unwrap(),
                "    Hello World[1]

    ┄No refs before this headline

[1]: http://example.com/world

┄But before this\n"
            )
        }

        #[test]
        fn flush_ref_links_at_end() {
            assert_eq!(
                render(
                    "Hello [World](http://example.com/world)

# Headline

Hello [Donald](http://example.com/Donald)"
                )
                .unwrap(),
                "Hello World[1]

[1]: http://example.com/world

┄Headline

Hello Donald[2]

[2]: http://example.com/Donald\n"
            )
        }
    }

    mod disabled_features {
        use anyhow::Result;
        use pretty_assertions::assert_eq;
        use syntect::parsing::SyntaxSet;

        use crate::terminal::TerminalProgram;
        use crate::*;

        use super::render_string;

        fn render(markup: &str) -> Result<String> {
            render_string(
                markup,
                &Settings {
                    #[cfg(feature = "render-image")]
                    resource_access: ResourceAccess::LocalOnly,
                    syntax_set: SyntaxSet::default(),
                    terminal_capabilities: TerminalProgram::Dumb.capabilities(),
                    terminal_size: TerminalSize::default(),
                },
            )
        }

        #[test]
        #[allow(non_snake_case)]
        fn GH_155_do_not_choke_on_footnoes() {
            assert_eq!(
                render(
                    "A footnote [^1]

[^1: We do not support footnotes."
                )
                .unwrap(),
                "A footnote [^1]

[^1: We do not support footnotes.\n"
            )
        }
    }
}
