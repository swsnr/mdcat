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

#[cfg(feature = "resources")]
use url::Url;

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

#[cfg(feature = "resources")]
impl ResourceAccess {
    /// Whether the resource access permits access to the given `url`.
    pub fn permits(self, url: &Url) -> bool {
        match self {
            ResourceAccess::LocalOnly if is_local(url) => true,
            ResourceAccess::RemoteAllowed => true,
            _ => false,
        }
    }
}

/// Whether `url` is readable as local file:.
#[cfg(feature = "resources")]
fn is_local(url: &Url) -> bool {
    url.scheme() == "file" && url.to_file_path().is_ok()
}

#[cfg(feature = "resources")]
pub fn read_url(url: &Url) -> std::io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{Error, ErrorKind};

    match url.scheme() {
        "file" => match url.to_file_path() {
            Ok(path) => {
                let mut buffer = Vec::new();
                File::open(path)?.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
            Err(_) => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Remote file: URL {} not supported", url),
            )),
        },
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Protocol of URL {} not supported", url),
        )),
    }
}

// /// A non-200 status code from a HTTP request.
// #[derive(Debug, Fail)]
// #[fail(
//     display = "Url {} failed with status code {}",
//     url,
//     status_code
// )]
// #[cfg(feature = "remote_resources")]
// pub struct HttpStatusError {
//     /// The URL that was requested
//     url: Url,
//     /// The status code.
//     status_code: reqwest::StatusCode,
// }

// impl<'a> Resource<'a> {

//     /// Read the contents of this resource.
//     ///
//     /// Supports local files and HTTP(S) resources.  `access` denotes the access
//     /// permissions.
//     pub fn read(&self, access: ResourceAccess) -> Result<Vec<u8>, Error> {
//         if self.may_access(access) {
//             match *self {
//                 Resource::Remote(ref url) => read_http(url),
//                 Resource::LocalFile(ref path) => {
//                     let mut buffer = Vec::new();
//                     File::open(path)?.read_to_end(&mut buffer)?;
//                     Ok(buffer)
//                 }
//             }
//         } else {
//             Err(io::Error::new(
//                 io::ErrorKind::PermissionDenied,
//                 "Remote resources not allowed",
//             ).into())
//         }
//     }
// }

// /// Read a resource from HTTP(S).
// #[cfg(feature = "remote_resources")]
// fn read_http(url: &Url) -> Result<Vec<u8>, Error> {
//     // We need to clone "Url" here because for some reason `get`
//     // claims ownership of Url which we don't have here.
//     let mut response = reqwest::get(url.clone())?;
//     if response.status().is_success() {
//         let mut buffer = Vec::new();
//         response.read_to_end(&mut buffer)?;
//         Ok(buffer)
//     } else {
//         Err(HttpStatusError {
//             url: url.clone(),
//             status_code: response.status(),
//         }.into())
//     }
// }

// #[cfg(not(feature = "remote_resources"))]
// fn read_http(_url: &Url) -> Result<Vec<u8>, Error> {
//     Err(NotSupportedError {
//         what: "remote resources",
//     }.into())
// }

#[cfg(all(test, feature = "resources"))]
mod tests {
    use super::*;

    #[test]
    fn resource_access_permits_local_resource() {
        let resource = Url::parse("file:///foo/bar").unwrap();
        assert!(ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    #[test]
    fn resource_access_permits_remote_file_url() {
        let resource = Url::parse("file://example.com/foo/bar").unwrap();
        assert!(!ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    #[test]
    fn resource_access_permits_https_url() {
        let resource = Url::parse("https:///foo/bar").unwrap();
        assert!(!ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    // mod read {
    //     use super::*;
    //     use std::error::Error;

    //     #[test]
    //     fn remote_resource_fails_with_permission_denied_without_access() {
    //         let resource = Resource::Remote(
    //             "https://eu.httpbin.org/bytes/100"
    //                 .parse()
    //                 .expect("No valid URL"),
    //         );
    //         let result = resource.read(ResourceAccess::LocalOnly);
    //         assert!(result.is_err(), "Unexpected success: {:?}", result);
    //         let error = match result.unwrap_err().downcast::<io::Error>() {
    //             Ok(e) => e,
    //             Err(error) => panic!("Not an IO error: {:?}", error),
    //         };

    //         assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    //         assert_eq!(error.description(), "Remote resources not allowed");
    //     }

    //     #[cfg(feature = "remote_resources")]
    //     #[test]
    //     fn remote_resource_fails_when_status_404() {
    //         let url: Url = "https://eu.httpbin.org/status/404"
    //             .parse()
    //             .expect("No valid URL");
    //         let resource = Resource::Remote(url.clone());
    //         let result = resource.read(ResourceAccess::RemoteAllowed);
    //         assert!(result.is_err(), "Unexpected success: {:?}", result);
    //         let error = result
    //             .unwrap_err()
    //             .downcast::<HttpStatusError>()
    //             .expect("Not an IO error");
    //         assert_eq!(error.status_code, reqwest::StatusCode::NOT_FOUND);
    //         assert_eq!(error.url, url);
    //     }

    //     #[cfg(feature = "remote_resources")]
    //     #[test]
    //     fn remote_resource_returns_content_when_status_200() {
    //         let resource = Resource::Remote(
    //             "https://eu.httpbin.org/bytes/100"
    //                 .parse()
    //                 .expect("No valid URL"),
    //         );
    //         let result = resource.read(ResourceAccess::RemoteAllowed);
    //         assert!(result.is_ok(), "Unexpected error: {:?}", result);
    //         assert_eq!(result.unwrap().len(), 100);
    //     }

    //     #[cfg(not(feature = "remote_resources"))]
    //     #[test]
    //     fn remote_resource_returns_not_supported_if_feature_is_disabled() {
    //         let resource = Resource::Remote(
    //             "https://eu.httpbin.org/bytes/100"
    //                 .parse()
    //                 .expect("No valid URL"),
    //         );
    //         let result = resource.read(ResourceAccess::RemoteAllowed);
    //         assert!(result.is_err(), "Unexpected success: {:?}", result);
    //         let error = result
    //             .unwrap_err()
    //             .downcast::<NotSupportedError>()
    //             .expect("Not a NotSupportedError!");
    //         assert_eq!(
    //             error,
    //             NotSupportedError {
    //                 what: "remote resources"
    //             }
    //         );
    //     }
    // }
}
