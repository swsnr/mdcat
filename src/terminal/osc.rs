// Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! OSC commands on terminals.

use std::io::{Result, Write};

#[cfg(feature = "osc8_links")]
use url::{Host, Url};

/// Write an OSC `command` to this terminal.
pub fn write_osc<W: Write>(writer: &mut W, command: &str) -> Result<()> {
    writer.write_all(&[0x1b, 0x5d])?;
    writer.write_all(command.as_bytes())?;
    writer.write_all(&[0x07])?;
    Ok(())
}

#[cfg(feature = "osc8_links")]
pub struct OSC8Links {
    hostname: String,
}

/// Whether the given `url` needs to get an explicit host.
///
/// [OSC 8] links require that `file://` URLs give an explicit hostname, as
/// received by [gethostname], to disambiguate `file://` printed over SSH
/// connections.
///
/// This function checks whether we need to explicit set the host of the given
/// `url` to the hostname of this system per `gethostname()`.  We do so if `url`
/// is a `file://` URL and the host is
///
/// * empty,
/// * `localhost`,
/// * or a IPv4/IPv6 loopback address.
///
/// [OSC 8]: https://git.io/vd4ee
/// [gethostname]: https://github.com/lunaryorn/gethostname.rs
#[cfg(feature = "osc8_links")]
fn url_needs_explicit_host(url: &Url) -> bool {
    if url.scheme() == "file" {
        match url.host() {
            None => true,
            Some(Host::Domain("localhost")) => true,
            Some(Host::Ipv4(addr)) if addr.is_loopback() => true,
            Some(Host::Ipv6(addr)) if addr.is_loopback() => true,
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(feature = "osc8_links")]
impl OSC8Links {
    /// Create OSC 8 links support for this host.
    ///
    /// Queries and remembers the hostname of this system as per `gethostname()`
    /// to resolve local `file://` URLs.
    pub fn for_localhost() -> OSC8Links {
        use gethostname::gethostname;
        OSC8Links {
            // Hostnames should be ASCII only anyway
            hostname: gethostname().to_string_lossy().into_owned(),
        }
    }

    /// Set a link to the given `destination` URL for subsequent text.
    ///
    /// Take ownership of `destination` to resolve `file://` URLs for localhost
    /// and loopback addresses, and print these with the proper hostname of the
    /// local system instead to make `file://` URLs work properly over SSH.
    ///
    /// See <https://git.io/vd4ee#file-uris-and-the-hostname>.
    pub fn set_link_url<W: Write>(&self, writer: &mut W, mut destination: Url) -> Result<()> {
        if url_needs_explicit_host(&destination) {
            destination.set_host(Some(&self.hostname)).unwrap();
        }
        self.set_link(writer, destination.as_str())
    }

    /// Clear the current link if any.
    pub fn clear_link<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.set_link(writer, "")
    }

    fn set_link<W: Write>(&self, writer: &mut W, destination: &str) -> Result<()> {
        write_osc(writer, &format!("8;;{}", destination))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "osc8_links")]
    fn url_needs_explicit_host() {
        let checks = [
            ("http://example.com/foo/bar", false),
            ("file:///foo/bar", true),
            ("file://localhost/foo/bar", true),
            ("file://127.0.0.1/foo/bar", true),
            ("file://[::1]/foo/bar", true),
        ];

        for (url, expected) in checks.iter() {
            let parsed = super::Url::parse(url).unwrap();
            let needs_host = super::url_needs_explicit_host(&parsed);
            assert_eq!(
                needs_host, *expected,
                "{:?} needs host? {}, but got {}",
                parsed, expected, needs_host
            );
        }
    }
}
