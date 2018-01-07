// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Write markdown to TTYs.

use std::io::{Result, Write};
use pulldown_cmark::Event;

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to
///
pub fn push_tty<'a, W, I>(writer: &mut W, events: I) -> Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    for event in events {
        write!(writer, "Event: {:?}\n", event)?;
    }
    Ok(())
}
