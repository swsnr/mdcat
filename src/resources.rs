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

use failure::Error;
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use url::Url;

// Required for remote resources
#[cfg(not(feature = "remote_resources"))]
use crate::error::NotSupportedError;
#[cfg(feature = "remote_resources")]
use reqwest;

/// What kind of resources mdcat may access when rendering.
///
/// This struct denotes whether mdcat shows inline images from remote URLs or
/// just from local files.
#[derive(Debug, Copy, Clone)]
pub enum ResourceAccess {
    /// Use only local files and prohibit remote resources.
    LocalOnly,
    /// Use local and remote resources alike.
    RemoteAllowed,
}

/// A resource referenced from a Markdown document.
pub enum Resource<'a> {
    /// A local file, referenced by its *absolute* path.
    LocalFile(Cow<'a, Path>),
    /// A remote resource, referenced by a URL.
    Remote(Url),
}

/// A non-200 status code from a HTTP request.
#[derive(Debug, Fail)]
#[fail(
    display = "Url {} failed with status code {}",
    url,
    status_code
)]
#[cfg(feature = "remote_resources")]
pub struct HttpStatusError {
    /// The URL that was requested
    url: Url,
    /// The status code.
    status_code: reqwest::StatusCode,
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

    /// Whether this resource is local.
    fn is_local(&self) -> bool {
        match *self {
            Resource::LocalFile(_) => true,
            _ => false,
        }
    }

    /// Whether we may access this resource under the given access permissions.
    pub fn may_access(&self, access: ResourceAccess) -> bool {
        match access {
            ResourceAccess::RemoteAllowed => true,
            ResourceAccess::LocalOnly => self.is_local(),
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
                .unwrap_or_else(|_| panic!(format!("Failed to join root URL with {:?}", path))),
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
    /// Supports local files and HTTP(S) resources.  `access` denotes the access
    /// permissions.
    pub fn read(&self, access: ResourceAccess) -> Result<Vec<u8>, Error> {
        if self.may_access(access) {
            match *self {
                Resource::Remote(ref url) => read_http(url),
                Resource::LocalFile(ref path) => {
                    let mut buffer = Vec::new();
                    File::open(path)?.read_to_end(&mut buffer)?;
                    Ok(buffer)
                }
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Remote resources not allowed",
            ).into())
        }
    }
}

/// Read a resource from HTTP(S).
#[cfg(feature = "remote_resources")]
fn read_http(url: &Url) -> Result<Vec<u8>, Error> {
    // We need to clone "Url" here because for some reason `get`
    // claims ownership of Url which we don't have here.
    let mut response = reqwest::get(url.clone())?;
    if response.status().is_success() {
        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer)?;
        Ok(buffer)
    } else {
        Err(HttpStatusError {
            url: url.clone(),
            status_code: response.status(),
        }.into())
    }
}

#[cfg(not(feature = "remote_resources"))]
fn read_http(_url: &Url) -> Result<Vec<u8>, Error> {
    Err(NotSupportedError {
        what: "remote resources",
    }.into())
}

#[cfg(test)]
mod tests {
    use super::Resource::*;
    use super::ResourceAccess::*;
    use super::*;
    use std::borrow::Cow::Borrowed;

    mod may_access {
        use super::*;

        #[test]
        fn local_resource() {
            let resource = LocalFile(Borrowed(Path::new("/foo/bar")));
            assert!(resource.may_access(LocalOnly));
            assert!(resource.may_access(RemoteAllowed));
        }

        #[test]
        fn remote_resource() {
            let resource = Remote("http://example.com".parse().unwrap());
            assert!(!resource.may_access(LocalOnly));
            assert!(resource.may_access(RemoteAllowed));
        }
    }

    mod into_url {
        use super::*;

        #[test]
        fn local_resource() {
            let resource = LocalFile(Borrowed(Path::new("/foo/bar")));
            assert_eq!(resource.into_url(), "file:///foo/bar".parse().unwrap());
        }

        #[test]
        fn remote_resource() {
            let url = "https://www.example.com/with/path?and&query"
                .parse::<Url>()
                .unwrap();
            assert_eq!(Remote(url.clone()).into_url(), url);
        }
    }

    mod local_path {
        use super::*;

        #[test]
        fn local_path_of_remote_resource() {
            let resource = Resource::Remote("http://example.com".parse().unwrap());
            assert_eq!(resource.local_path(), None);
        }

        #[test]
        fn local_path_of_file_url() {
            let resource = Resource::Remote("file:///spam/with/eggs".parse().unwrap());
            let path = resource.local_path();
            assert!(path.is_some());
            assert_eq!(path.unwrap(), Path::new("/spam/with/eggs"));
        }

        #[test]
        fn local_path_of_local_resource() {
            let path = Path::new("/foo/bar");
            let resource = Resource::LocalFile(Borrowed(path));
            assert_eq!(resource.local_path().unwrap(), path);
        }
    }

    mod read {
        use super::*;
        use std::error::Error;

        #[test]
        fn remote_resource_fails_with_permission_denied_without_access() {
            let resource = Resource::Remote(
                "https://eu.httpbin.org/bytes/100"
                    .parse()
                    .expect("No valid URL"),
            );
            let result = resource.read(ResourceAccess::LocalOnly);
            assert!(result.is_err(), "Unexpected success: {:?}", result);
            let error = match result.unwrap_err().downcast::<io::Error>() {
                Ok(e) => e,
                Err(error) => panic!("Not an IO error: {:?}", error),
            };

            assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            assert_eq!(error.description(), "Remote resources not allowed");
        }

        #[cfg(feature = "remote_resources")]
        #[test]
        fn remote_resource_fails_when_status_404() {
            let url: Url = "https://eu.httpbin.org/status/404"
                .parse()
                .expect("No valid URL");
            let resource = Resource::Remote(url.clone());
            let result = resource.read(ResourceAccess::RemoteAllowed);
            assert!(result.is_err(), "Unexpected success: {:?}", result);
            let error = result
                .unwrap_err()
                .downcast::<HttpStatusError>()
                .expect("Not an IO error");
            assert_eq!(error.status_code, reqwest::StatusCode::NOT_FOUND);
            assert_eq!(error.url, url);
        }

        #[cfg(feature = "remote_resources")]
        #[test]
        fn remote_resource_returns_content_when_status_200() {
            let resource = Resource::Remote(
                "https://eu.httpbin.org/bytes/100"
                    .parse()
                    .expect("No valid URL"),
            );
            let result = resource.read(ResourceAccess::RemoteAllowed);
            assert!(result.is_ok(), "Unexpected error: {:?}", result);
            assert_eq!(result.unwrap().len(), 100);
        }

        #[cfg(not(feature = "remote_resources"))]
        #[test]
        fn remote_resource_returns_not_supported_if_feature_is_disabled() {
            let resource = Resource::Remote(
                "https://eu.httpbin.org/bytes/100"
                    .parse()
                    .expect("No valid URL"),
            );
            let result = resource.read(ResourceAccess::RemoteAllowed);
            assert!(result.is_err(), "Unexpected success: {:?}", result);
            let error = result
                .unwrap_err()
                .downcast::<NotSupportedError>()
                .expect("Not a NotSupportedError!");
            assert_eq!(
                error,
                NotSupportedError {
                    what: "remote resources"
                }
            );
        }
    }
}
