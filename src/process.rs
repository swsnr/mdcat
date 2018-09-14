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

//! Tools for subprocesses.

use std::process::ExitStatus;

/// A process failed.
#[derive(Debug, Fail)]
#[fail(
    display = "Command {} failed with {}: {}",
    command,
    status,
    error
)]
pub struct ProcessError {
    /// The command that failed.
    pub command: String,
    /// The exit code of the failed command.
    pub status: ExitStatus,
    /// The error output of the command.
    pub error: String,
}
