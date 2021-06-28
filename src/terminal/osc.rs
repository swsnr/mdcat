// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! OSC commands on terminals.

use fehler::throws;
use std::io::{Error, Result, Write};

use url::{Host, Url};

/// Write an OSC `command` to this terminal.
#[throws]
pub fn write_osc<W: Write>(writer: &mut W, command: &str) -> () {
    writer.write_all(&[0x1b, 0x5d])?;
    writer.write_all(command.as_bytes())?;
    writer.write_all(&[0x07])?;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Osc8Links;

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
fn url_needs_explicit_host(url: &Url) -> bool {
    url.scheme() == "file"
        && match url.host() {
            None => true,
            Some(Host::Domain("localhost")) => true,
            Some(Host::Ipv4(addr)) if addr.is_loopback() => true,
            Some(Host::Ipv6(addr)) if addr.is_loopback() => true,
            _ => false,
        }
}

impl Osc8Links {
    /// Set a link to the given `destination` URL for subsequent text.
    ///
    /// Take ownership of `destination` to resolve `file://` URLs for localhost
    /// and loopback addresses, and print these with the proper `hostname` of the
    /// local system instead to make `file://` URLs work properly over SSH.
    ///
    /// See <https://git.io/vd4ee#file-uris-and-the-hostname>.
    pub fn set_link_url<W: Write>(
        &self,
        writer: &mut W,
        mut destination: Url,
        hostname: &str,
    ) -> Result<()> {
        if url_needs_explicit_host(&destination) {
            destination.set_host(Some(hostname)).unwrap();
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
