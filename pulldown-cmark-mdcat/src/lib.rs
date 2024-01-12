// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Write markdown to TTYs.
//!
//! See [`push_tty`] for the main entry point.
//!
//! ## MSRV
//!
//! This library generally supports only the latest stable Rust version.
//!
//! ## Features
//!
//! - `default` enables `svg`, `regex-fancy`, and `image-processing`.
//!
//! - `svg` includes support for rendering SVG images to PNG for terminals which do not support SVG
//!   images natively.  This feature adds a dependency on `resvg`.
//!
//! - `image-processing` enables processing of pixel images before rendering.  This feature adds
//!   a dependency on `image`.  If disabled mdcat will not be able to render inline images on some
//!   terminals, or render images incorrectly or at wrong sizes on other terminals.
//!
//!   Do not disable this feature unless you are sure that you won't use inline images, or accept
//!   incomplete rendering of images.  Please do not report issues with inline images with this
//!   feature disabled.
//!
//!   This feature only exists to allow building with minimal dependencies for use cases where
//!   inline image support is not used or required.  Do not disable this feature unless you know
//!   you won't use inline images, or can accept buggy inline image rendering.
//!
//!   Please **do not report bugs** about inline image rendering with this feature disabled, unless
//!   the issue can also be reproduced if the feature is enabled.
//!
//! - `regex-fancy` and `regex-onig` enable the corresponding features of the [`syntect`] crate,
//!   i.e. determine whether syntect uses the regex-fancy Rust crate or the Oniguruma C library as
//!   its regex engine.  The former is slower, but does not imply a native dependency, the latter
//!   is faster, but you need to compile and link a C library.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use pulldown_cmark::Event;
use syntect::parsing::SyntaxSet;
use tracing::instrument;
use url::Url;

use crate::bufferline::BufferLines;
pub use crate::resources::ResourceUrlHandler;
pub use crate::terminal::capabilities::TerminalCapabilities;
pub use crate::terminal::{TerminalProgram, TerminalSize};
pub use crate::theme::Theme;

mod references;
pub mod resources;
pub mod terminal;
mod theme;

pub mod bufferline;
pub mod markdown_widget;
mod reflow;
mod render;

/// Settings for markdown rendering.
#[derive(Debug)]
pub struct Settings<'a> {
    /// Capabilities of the terminal mdcat writes to.
    pub terminal_capabilities: TerminalCapabilities,
    /// The size of the terminal mdcat writes to.
    pub terminal_size: TerminalSize,
    /// Syntax set for syntax highlighting of code blocks.
    pub syntax_set: &'a SyntaxSet,
    /// Colour theme for mdcat
    pub theme: Theme,
}

/// The environment to render markdown in.
#[derive(Debug)]
pub struct Environment {
    /// The base URL to resolve relative URLs with.
    pub base_url: Url,
    /// The local host name.
    pub hostname: String,
}

#[cfg(windows)]
fn gethostname() -> OsString {
    gethostname::gethostname()
}

#[cfg(unix)]
fn gethostname() -> OsString {
    use std::os::unix::prelude::*;

    OsString::from_vec(rustix::system::uname().nodename().to_bytes().to_vec())
}

impl Environment {
    /// Create an environment for the local host with the given `base_url`.
    ///
    /// Take the local hostname from `gethostname`.
    pub fn for_localhost(base_url: Url) -> Result<Self> {
        gethostname()
            .into_string()
            .map_err(|raw| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("gethostname() returned invalid unicode data: {raw:?}"),
                )
            })
            .map(|hostname| Environment { base_url, hostname })
    }

    /// Create an environment for a local directory.
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
pub fn push_tty<'a, 'e, I>(
    settings: &Settings,
    environment: &Environment,
    resource_handler: &dyn ResourceUrlHandler,
    writer: &'a mut BufferLines,
    mut events: I,
) -> Result<()>
where
    I: Iterator<Item = Event<'e>>,
{
    use render::*;

    let StateAndData(final_state, final_data) = events.try_fold(
        StateAndData(State::default(), StateData::default()),
        |StateAndData(state, data), event| {
            write_event(
                writer,
                settings,
                environment,
                &resource_handler,
                state,
                data,
                event,
            )
        },
    )?;
    finish(writer, settings, environment, final_state, final_data)
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::Parser;

    use crate::resources::NoopResourceHandler;

    use super::*;

    fn render_string(input: &str, settings: &Settings) -> Result<String> {
        let source = Parser::new(input);
        let mut sink = Vec::new();
        let env =
            Environment::for_local_directory(&std::env::current_dir().expect("Working directory"))?;
        push_tty(settings, &env, &NoopResourceHandler, &mut sink, source)?;
        Ok(String::from_utf8_lossy(&sink).into())
    }

    mod layout {
        use similar_asserts::assert_eq;
        use syntect::parsing::SyntaxSet;

        use crate::terminal::TerminalProgram;
        use crate::*;

        use super::render_string;

        fn render(markup: &str) -> Result<String> {
            render_string(
                markup,
                &Settings {
                    syntax_set: &SyntaxSet::default(),
                    terminal_capabilities: TerminalProgram::Dumb.capabilities(),
                    terminal_size: TerminalSize::default(),
                    theme: Theme::default(),
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
        use similar_asserts::assert_eq;
        use syntect::parsing::SyntaxSet;

        use crate::terminal::TerminalProgram;
        use crate::*;

        use super::render_string;

        fn render(markup: &str) -> Result<String> {
            render_string(
                markup,
                &Settings {
                    syntax_set: &SyntaxSet::default(),
                    terminal_capabilities: TerminalProgram::Dumb.capabilities(),
                    terminal_size: TerminalSize::default(),
                    theme: Theme::default(),
                },
            )
        }

        #[test]
        #[allow(non_snake_case)]
        fn GH_155_do_not_choke_on_footnotes() {
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
