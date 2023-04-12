// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Terminal utilities.

// Support modules for terminal writing.

mod ansi;
mod osc;
mod size;

pub mod capabilities;
mod detect;

pub use self::ansi::AnsiStyle;
pub use self::detect::TerminalProgram;
pub use self::size::TerminalSize;
