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

use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::fs::File;
use std::borrow::Cow;
use std::path::Path;
use url::Url;

/// A resource referenced from a Markdown document.
pub enum Resource<'a> {
    /// A local file, referenced by its *absolute* path.
    LocalFile(Cow<'a, Path>),
    /// A remote resource, referenced by a URL.
    Remote(Url),
}

impl<'a> Resource<'a> {
    /// Obtain a resource from a markdown `reference`.
    ///
    /// Try to parse `reference` as a URL.  If this succeeds assume that
    /// `reference` refers to a remote resource and return a `Remote` resource.
    ///
    /// Otherwise assume that `reference` denotes a local file by its path and
    /// return a `LocalFile` resource.  If `reference` holds a relative path
    /// join it against `base_dir` first.
    pub fn from_reference(base_dir: &Path, reference: &'a str) -> Resource<'a> {
        if let Ok(url) = Url::parse(reference) {
            Resource::Remote(url)
        } else {
            let path = Path::new(reference);
            if path.is_absolute() {
                Resource::LocalFile(Cow::Borrowed(path))
            } else {
                Resource::LocalFile(Cow::Owned(base_dir.join(path)))
            }
        }
    }

    /// Convert this resource into a URL.
    ///
    /// Return a `Remote` resource as is, and a `LocalFile` as `file:` URL.
    pub fn into_url(self) -> Url {
        match self {
            Resource::Remote(url) => url,
            Resource::LocalFile(path) => Url::parse("file:///")
                .expect("Failed to parse file root URL!")
                .join(&path.to_string_lossy())
                .expect(&format!("Failed to join root URL with {:?}", path)),
        }
    }

    /// Extract the local path from this resource.
    ///
    /// If the resource is a `LocalFile`, or a `file://` URL pointing to a local
    /// file return the local path, otherwise return `None`.
    pub fn local_path(&'a self) -> Option<Cow<'a, Path>> {
        match *self {
            Resource::Remote(ref url) if url.scheme() == "file" && url.host().is_none() => {
                Some(Cow::Borrowed(Path::new(url.path())))
            }
            Resource::LocalFile(ref path) => Some(Cow::Borrowed(path)),
            _ => None,
        }
    }

    /// Convert this resource to a string.
    ///
    /// For local resource return the lossy UTF-8 representation of the path,
    /// for remote resource the string serialization of the URL.
    pub fn as_str(&'a self) -> Cow<'a, str> {
        match *self {
            Resource::Remote(ref url) => Cow::Borrowed(url.as_str()),
            Resource::LocalFile(ref path) => path.to_string_lossy(),
        }
    }

    /// Read the contents of this resource.
    ///
    /// Currently only supports local files; remote resources return a
    /// permission denied error.
    pub fn read(&self) -> Result<Vec<u8>> {
        match *self {
            Resource::Remote(_) => Err(Error::new(
                ErrorKind::PermissionDenied,
                "Remote resources not allowed",
            )),
            Resource::LocalFile(ref path) => {
                let mut buffer = Vec::new();
                let mut source = File::open(path)?;
                source.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
        }
    }
}
