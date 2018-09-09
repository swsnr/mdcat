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

//! Terminal errors.

use failure::Error;

/// The terminal does not support something.
#[derive(Debug, Fail)]
#[fail(display = "This terminal does not support {}.", what)]
pub struct NotSupportedError {
    /// The operation which the terminal did not support.
    pub what: &'static str,
}

/// Ignore a `NotSupportedError`.
pub trait IgnoreNotSupported {
    /// The type after ignoring `NotSupportedError`.
    type R;

    /// Elide a `NotSupportedError` from this value.
    fn ignore_not_supported(self) -> Self::R;
}

impl IgnoreNotSupported for Error {
    type R = Result<(), Error>;

    fn ignore_not_supported(self) -> Self::R {
        self.downcast::<NotSupportedError>().map(|_| ())
    }
}

impl IgnoreNotSupported for Result<(), Error> {
    type R = Result<(), Error>;

    fn ignore_not_supported(self) -> Self::R {
        self.or_else(|err| err.ignore_not_supported())
    }
}
