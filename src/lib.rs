// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Command line application to render markdown to TTYs.
//!
//! Note that as of version 2.0.0 mdcat itself no longer contains the core rendering functions.
//! Use [`pulldown-cmark-mdcat`] instead.

#![deny(warnings, missing_docs, clippy::all)]

use std::fs::File;
use std::io::{prelude::*, BufWriter};
use std::io::{stdin, Result};
use std::path::PathBuf;

use pulldown_cmark::{Options, Parser};
use pulldown_cmark_mdcat::resources::{
    DispatchingResourceHandler, FileResourceHandler, HttpResourceHandler, ResourceUrlHandler,
};
use pulldown_cmark_mdcat::{Environment, Settings};
use tracing::{event, instrument, Level};

use args::ResourceAccess;
use output::Output;

/// Argument parsing for mdcat.
#[allow(missing_docs)]
pub mod args;
/// Output handling for mdcat.
pub mod output;

/// Default read size limit for resources.
pub static DEFAULT_RESOURCE_READ_LIMIT: u64 = 104_857_600;

/// Read input for `filename`.
///
/// If `filename` is `-` read from standard input, otherwise try to open and
/// read the given file.
pub fn read_input<T: AsRef<str>>(filename: T) -> Result<(PathBuf, String)> {
    let cd = std::env::current_dir()?;
    let mut buffer = String::new();

    if filename.as_ref() == "-" {
        stdin().read_to_string(&mut buffer)?;
        Ok((cd, buffer))
    } else {
        let mut source = File::open(filename.as_ref())?;
        source.read_to_string(&mut buffer)?;
        let base_dir = cd
            .join(filename.as_ref())
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(cd);
        Ok((base_dir, buffer))
    }
}

/// Process a single file.
///
/// Read from `filename` and render the contents to `output`.
#[instrument(skip(output, settings, resource_handler), level = "debug")]
pub fn process_file(
    filename: &str,
    settings: &Settings,
    resource_handler: &dyn ResourceUrlHandler,
    output: &mut Output,
) -> Result<()> {
    let (base_dir, input) = read_input(filename)?;
    event!(
        Level::TRACE,
        "Read input, using {} as base directory",
        base_dir.display()
    );
    let parser = Parser::new_ext(
        &input,
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    );
    let env = Environment::for_local_directory(&base_dir)?;

    let mut sink = BufWriter::new(output.writer());
    pulldown_cmark_mdcat::push_tty(settings, &env, resource_handler, &mut sink, parser)
        .and_then(|_| {
            event!(Level::TRACE, "Finished rendering, flushing output");
            sink.flush()
        })
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::BrokenPipe {
                event!(Level::TRACE, "Ignoring broken pipe");
                Ok(())
            } else {
                event!(Level::ERROR, ?error, "Failed to process file: {:#}", error);
                Err(error)
            }
        })
}

/// Create the resource handler for mdcat.
pub fn create_resource_handler(access: ResourceAccess) -> Result<DispatchingResourceHandler> {
    let mut resource_handlers: Vec<Box<dyn ResourceUrlHandler>> = vec![Box::new(
        FileResourceHandler::new(DEFAULT_RESOURCE_READ_LIMIT),
    )];
    if let ResourceAccess::Remote = access {
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        event!(
            target: "mdcat::main",
            Level::DEBUG,
            "Remote resource access permitted, creating HTTP client with user agent {}",
            user_agent
        );
        resource_handlers.push(Box::new(HttpResourceHandler::with_user_agent(
            DEFAULT_RESOURCE_READ_LIMIT,
            user_agent,
        )?));
    }
    Ok(DispatchingResourceHandler::new(resource_handlers))
}
