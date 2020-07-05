// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Provide utilities for references.

use crate::Environment;
use url::Url;

/// A base to resolve references with.
pub trait UrlBase {
    /// Resolve the given `reference` against self.
    fn resolve_reference(&self, reference: &str) -> Option<Url>;
}

impl UrlBase for Url {
    /// Resolve a reference against this URL.
    fn resolve_reference(&self, reference: &str) -> Option<Url> {
        Url::parse(reference).or_else(|_| self.join(reference)).ok()
    }
}

impl UrlBase for Environment {
    /// Resolve a reference against the `base_url` of this environment.
    fn resolve_reference(&self, reference: &str) -> Option<Url> {
        self.base_url.resolve_reference(reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use url::Url;

    #[test]
    fn absolute_url() {
        let url = Url::parse("file:///some/root/")
            .unwrap()
            .resolve_reference("http://www.example.com/reference");
        assert_eq!(
            url.as_ref().map_or("", |u| u.as_str()),
            "http://www.example.com/reference"
        );
    }

    #[test]
    fn relative_path() {
        let url = Url::parse("file:///some/root/")
            .unwrap()
            .resolve_reference("./foo.md");

        assert_eq!(
            url.as_ref().map_or("", |u| u.as_str()),
            "file:///some/root/foo.md"
        );
    }

    #[test]
    fn absolute_path() {
        let url = Url::parse("file:///some/root/")
            .unwrap()
            .resolve_reference("/foo.md");
        assert_eq!(url.as_ref().map_or("", |u| u.as_str()), "file:///foo.md");
    }

    #[test]
    fn base_with_drive_letter_and_absolute_path() {
        let url = Url::parse("file:///d:/some/folder")
            .unwrap()
            .resolve_reference("/foo");
        assert_eq!(url.as_ref().map_or("", |u| u.as_str()), "file:///d:/foo");
    }

    #[test]
    fn base_with_drive_letter_and_root_path() {
        let url = Url::parse("file:///d:/some/folder")
            .unwrap()
            .resolve_reference("/");
        assert_eq!(url.as_ref().map_or("", |u| u.as_str()), "file:///d:/");
    }
}
