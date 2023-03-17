// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! mdcat as a library.
//!
//! This library is provided for compatibility with consumers of the previous mdcat API.
//! Please transition to the [pulldown-cmark-tty crate](https://docs.rs/pulldown-cmark-tty)
//! instead.

pub use pulldown_cmark_tty::*;
