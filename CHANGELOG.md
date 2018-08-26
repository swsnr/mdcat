# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1] – 2018-08-26
### Added
- Publish binaries for Linux, macOS and Windows (see [GH-28][]).

### Fixed
- Correctly build macOS and Linux binaries on Travis CI.

[GH-28]: https://github.com/lunaryorn/mdcat/issues/28

## [0.9.0] – 2018-08-26
### Added
- `mdcat` builds on Windows now (see [GH-33][] and [GH-34][]).

### Changed
- Refactor internal terminal representation, replacing the terminal enum with a
  new `Terminal` trait and dynamic dispatch (see [GH-35][]).
- Allow to disable specific terminal backends (see [GH-35][]).
- Update minimum Rust version to 1.27.

[GH-33]: https://github.com/lunaryorn/mdcat/pull/33
[GH-34]: https://github.com/lunaryorn/mdcat/pull/34
[GH-35]: https://github.com/lunaryorn/mdcat/pull/35

## [0.8.0] – 2018-02-15
### Added
- Render SVG images in iTerm2 with `rsvg-convert` (requires `librsvg`).
- Expose `TerminalWrite` in `mdcat` crate (see [GH-20][]).

[GH-20]: https://github.com/lunaryorn/mdcat/pull/20

## [0.7.0] – 2018-02-08
### Added
- Show images from HTTP and HTTPS URLs inline in iTerm2.
- Add `--local` flag to render only local images inline; for remote images, eg,
  HTTP URLs, show only the image title and the URL.
- Expose `mdcat` as a library crate (see [GH-18][]), but with no guarantees
  about a stable interface, as `mdcat` stays at version 0.x for now.
- Show `--help` with colours.

### Changed
- Adhere to Semantic Versioning, but stay pre-1.0 so anything still goes.

[GH-18]: https://github.com/lunaryorn/mdcat/issues/18

## [0.6.0] – 2018-02-02
### Added
- Show inline images in [Terminology][] (see [GH-16][]).

[Terminology]: http://terminolo.gy
[GH-16]: https://github.com/lunaryorn/mdcat/pull/16

### Changed
- Improve `--help` output: Hide some redundant options, add a bug reporting URL
  and explain the purpose of `mdcat`.
- Reduce dependencies and thus build time

## [0.5.0] – 2018-01-27
### Added
- Show links inline in iTerm2 and terminals based on VTE 0.50 or newer (see
  [GH-8][], [GH-14][] and [GH-15][]).

[GH-8]: https://github.com/lunaryorn/mdcat/issues/8
[GH-14]: https://github.com/lunaryorn/mdcat/issues/14
[GH-15]: https://github.com/lunaryorn/mdcat/issues/15

### Changed
- Improve `--help` output.

### Fixed
- Remove redundant default value from `--colour` help text (see [GH-10][]).
- Replace light black with green; the former doesn't work with Solarized Dark.

[GH-10]: https://github.com/lunaryorn/mdcat/pull/10

## [0.4.0] – 2018-01-21
### Changed
- Use 8-bit ANSI colours for syntax highlighting to fit all kinds of terminal
  colour themes.

### Fixed
- Remove excess space at the end of code blocks

### Removed
- Remove `--light` switch which became redundant due to better syntax
  highlighting.

## [0.3.0] – 2018-01-19
### Added
- Print image links
- Show images inline on iTerm.

### Changed
- Rename to `mdcat`; I have no plans to add paging to this tool.

## [0.2.0] – 2018-01-16
### Added
- Highlight code blocks with Solarized color theme (light or dark).
- Naively show inline and block HTML.
- Set iTerm marks for headings.
- Auto-detect whether mdless can use iTerm2 marks.
- Add `--colour` flag to enable or disable coloured output.

## [0.1.1] – 2018-01-14
### Fixed

- Fix Travis CI badge on crates.io.
- Fix license layout in README.

## [0.1] – 2018-01-14
### Added

- Support inline formatting.
- Support headings.
- Support code blocks and block quotes.
- Support ordered and unordered lists, with nest.
- Show links, with references grouped by section.
