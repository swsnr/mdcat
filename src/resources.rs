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

//! Access to resources referenced from markdown documents.

use std::path::Path;
use url::Url;

/// Get a URL from a path in a Markdown file.
///
/// If we can parse `path` as URL return the corresponding URL.  Otherwise we
/// consider `path` as a local file system path, and turn it into a absolute
/// `file://` URL resolved against the given `base_dir`.
pub fn url_from_path(base_dir: &Path, path: &str) -> Url {
    Url::parse(path).unwrap_or_else(|_| {
        //
        let absolute_path = base_dir.join(path);
        Url::parse("file:///")
            .expect("Failed to parse file root URL!")
            .join(&absolute_path.to_string_lossy())
            .expect(&format!("Failed to join root URL with {:?}", path))
    })
}
