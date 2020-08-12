# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

To publish a new release run `scripts/release` from the project directory.

## [Unreleased]

## [0.21.0] – 2020-08-12

### Added
- Add `--paginate` flag to paginate the output of mdcat (see [GH-152]);
  with this flag mdcat sends it output to `less -R` or any alternative pager set in `$MDCAT_PAGER` or `$PAGER`.
  This flag also disables all terminal-specific formatting as the pager likely won't support it.
- Paginate output by default if invoked as `mdless`, that is, if `mdcat` is hard-linked to `mdless`.

### Fixed
- Render email autolinks (i.e. `<hello@example.com>`) as `mailto:` links.

[GH-152]: https://github.com/lunaryorn/mdcat/issues/152

## [0.20.0] – 2020-07-05
### Added
- `mdcat::TerminalCapabilities` now exposes constructors for specific terminal emulators.
- Render reference link definitions as inline links if possible (see [GH-149]).  
    This mainly affects image links inside inline links which get rendered as reference links.
- `mdcat::Environment` now contains all environment information required to render properly, namely the local hostname and the base URL.

### Changed
- `mdcat::push_tty` now takes an `mdcat::Environment` instead of `base_dir`.
    `base_dir` is now part of `mdcat::Environment`.
- Image links now use purple foreground text (see [GH-140] and [GH-149]).
- Image links render as inline links if the terminal does not support inline images and the image is not inside another link (see [GH-141]).
- `mdcat::TerminalCapabilities` now uses `Option` to denote missing capabilities.

### Fixed
- Always treat links targets as URLs, never as paths.
- On ITerm2 only use the last segment of image URLs as filename for inline images (see [GH-149]).
    Previously mdcat used the full URL based on a misunderstanding of the [Inline Images Protocol].

[GH-140]: https://github.com/lunaryorn/mdcat/issues/140
[GH-141]: https://github.com/lunaryorn/mdcat/issues/141
[GH-149]: https://github.com/lunaryorn/mdcat/issues/149
[Inline Images Protocol]: https://iterm2.com/documentation-images.html

## [0.19.0] – 2020-06-19
### Added
- Release packages now include generated shell completions for Bash, Zsh and Fish.

### Changed
- Blockquotes no longer have green foreground text (see [GH-144]).

[GH-144]: https://github.com/lunaryorn/mdcat/issues/144

## [0.18.4] – 2020-06-14
### Fixed
- Fix typo in release workflow.
- Update all dependencies to no longer depend on yanked crate versions.

## [0.18.3] – 2020-06-14
### Fixed
- Properly ignore alt text of inline images (see [GH-148]).

[GH-148]: https://github.com/lunaryorn/mdcat/issues/148

## [0.18.2] – 2020-05-31
### Fixed
- Properly upload binaries for releases.

## [0.18.1] – 2020-05-31
### Fixed
- Fix typo in release workflow.

## [0.18.0] – 2020-05-31
### Added 
- Add `mdcat::Error` as type alias to `std::io::Error`.

### Changed
- New simpler rendering algorithm (see GH-[142]) which solves numerous rendering
  issues (see below).
- Handle internal errors with [anyhow] to add more context to errors (see
  [GH-139]}.
- `mdcat::push_tty` only fails with `std::io::Error`: `mdcat` never visibly
  fails unless it can’t write output.

### Fixed
- Respect `--local-only` and resource access policy; this got lost in some
  refactoring (see [GH-146]).
- Consistent margins and newlines around paragraphs, HTML blocks and inside list
  items (see [GH-142]).
- Correctly indent nested code blocks in lists and block quotes (see [GH-142]).
- No longer print leading blank lines before lists (see [GH-142]).
- Correctly indent block quotes (see [GH-142]).
- Colorize the entire text of links (see [GH-142]).

[GH-142]: https://github.com/lunaryorn/mdcat/issues/142
[GH-139]: https://github.com/lunaryorn/mdcat/issues/139
[anyhow]: https://docs.rs/crate/anyhow
[GH-146]: https://github.com/lunaryorn/mdcat/issues/146

## [0.17.1] – 2020-05-24
### Fixed
- Correctly scale down large images on [kitty] (see [GH-124] and [GH-133] by
  [@fspillner]).

[GH-124]: https://github.com/lunaryorn/mdcat/issues/124
[GH-133]: https://github.com/lunaryorn/mdcat/pull/133

## [0.17.0] – 2020-05-20
### Changed
- `mdcat` is now distributed under the [MPL 2](http://mozilla.org/MPL/2.0/) license;
  some source files remain Apache 2.0 due to 3rd party rights (see [GH-138]).

### Fixed
- Do not fail with broken pipe error when rending large images (see [GH-134] by
  [@fspillner]).

[GH-134]: https://github.com/lunaryorn/mdcat/issues/134  
[GH-138]: https://github.com/lunaryorn/mdcat/issues/138

## [0.16.1] – 2020-05-15
### Changed
- `mdcat::push_tty` now takes a `mdcat::Settings` struct which groups all
  external settings.

### Fixed
- Ignore broken pipes; `mdcat | head` no longer errors when `head` closes stdout
  of mdcat early (see [GH-136]).

[GH-136]: https://github.com/lunaryorn/mdcat/issues/136

## [0.16.0] – 2020-04-11
### Changed
- Upgrade to syntect 4.1 and enable its pure Rust regex backend to simplify
  building (see [GH-131]).  This crate now builds without Clang which fixes
  Clang-related build issues (see [GH-90])

[GH-90]: https://github.com/lunaryorn/mdcat/issues/90
[GH-131]: https://github.com/lunaryorn/mdcat/pull/131

## [0.15.1] – 2020-02-15
### Changed
- Update pulldown-cmark to 0.7.

## [0.15.0] – 2020-01-11
### Added
- Release builds now perform full link-time optimization to create a smaller
  binary.  We do recommend to also `strip` the `mdcat` binary.
- Render SVG images in [kitty] (see [GH-114]).
- Update to reqwest 0.10.
- Process file list as input (see [GH-54] and [GH-115], by [@norman-abramovitz]):
    - Add `--fail` flag to exit on the first error when processing a file list;
      the default behaviour is to continue with the next file in case of error.

### Changed
- Replace `remote_resources` feature with `reqwest` feature to use reqwest for
  retrieving remote resources, and fall back to the `curl` command if `reqwest`
  is disabled.

[GH-114]: https://github.com/lunaryorn/mdcat/pull/114
[GH-115]: https://github.com/lunaryorn/mdcat/pull/115
[GH-54]: https://github.com/lunaryorn/mdcat/issues/54
[@norman-abramovitz]: https://github.com/norman-abramovitz

## [0.14.0] – 2019-12-18
### Added
- Render images in [kitty] (see [GH-65] and [GH-104] by [@fspillner]).

### Changed
- Update pulldown-cmark to 0.6 which supports CommonMark 0.29 and improves
  parser speed and correctness.
- Enable SIMD in pulldown-cmark to squeeze out the last bit of performance.
- Remove all features except `remote_resources` to reduce build complexity.

### Removed
- No longer depend on `immeta`.

[kitty]: https://sw.kovidgoyal.net/kitty/
[GH-65]: https://github.com/lunaryorn/mdcat/issues/65
[GH-84]: https://github.com/lunaryorn/mdcat/issues/84
[GH-104]: https://github.com/lunaryorn/mdcat/pull/104
[@fspillner]: https://github.com/fspillner

## [0.13.0] – 2019-03-30
### Added
- Render task lists nicely (see [GH-72]).
- Render strike-through text (see [GH-71]).  Some terminals do not support this
  feature, and mdcat does not have a fallback currently (see [GH-73]).

[GH-73]: https://github.com/lunaryorn/mdcat/issues/73
[GH-72]: https://github.com/lunaryorn/mdcat/issues/72
[GH-71]: https://github.com/lunaryorn/mdcat/issues/71

## [0.12.1] – 2018-12-24
### Fixed
- Do not add newline after inline text with styles disabled (see [GH-49]).

[GH-49]: https://github.com/lunaryorn/mdcat/issues/49

## [0.12.0] – 2018-12-20
### Added
- Add `TerminalCapability` struct as replacement for `mdcat::Terminal` trait to
  remove dynamic dispatch and allow for more accurate and less complicated
  conditional compilation of terminal support for different platforms (see
  [GH-45]).
- Move to Rust 2018, and raise minimum supported Rust version to 1.31 (see
  [GH-46]).

### Changed
- Drop support for Rust 1.29 and older.
- Do not test specific Rust on versions on Travis CI any longer; Rust stable
  becomes the lowest supported Rust version.

### Removed
- `mdcat::Terminal` trait and implementations (see [GH-45]).

### Fixed
- Set hostname to local hostname for inline links to `file://` URLs, which
  should properly resolve `file://` URLs over SSH (see [OSC 8 file URLs],
  [GH-42] and [GH-44]).

[OSC 8 file URLs]: https://git.io/vd4ee#file-uris-and-the-hostname
[GH-42]: https://github.com/lunaryorn/mdcat/pull/42
[GH-44]: https://github.com/lunaryorn/mdcat/pull/44
[GH-45]: https://github.com/lunaryorn/mdcat/pull/45
[GH-46]: https://github.com/lunaryorn/mdcat/issues/46

## [0.11.0] – 2018-10-25
### Changed
- Always print colours regardless of whether stdout if a tty or not.
- Replace `--colour` option with a `--no-colour` flag to turn off styled output.
- `mdcat::push_tty` no longer takes ownership of the `terminal` argument (see
  [GH-41]).
- Travis CI builds Windows binaries now.
- Test formatting output.

[GH-41]: https://github.com/lunaryorn/mdcat/issues/41

## [0.10.1] – 2018-09-09
### Fixed
- Properly package musl binary on Travis CI; restores Linux binary in releases.

## [0.10.0] – 2018-09-09
### Added
- Support colours on Windows 10 console (see [GH-36]).
- Support musl target on Linux (see [GH-37] and [GH-38]).
- Published Linux binary statically links musl now, and has no runtime
  dependencies (see [GH-29] and [GH-38]).

[GH-29]: https://github.com/lunaryorn/mdcat/issues/29
[GH-36]: https://github.com/lunaryorn/mdcat/pull/36
[GH-37]: https://github.com/lunaryorn/mdcat/issues/37
[GH-38]: https://github.com/lunaryorn/mdcat/pull/38

## [0.9.2] – 2018-08-26
### Fixed
- Do not falsely ignore deployments from Travis CI for Linux and macOS.

## [0.9.1] – 2018-08-26
### Added
- Publish binaries for Linux, macOS and Windows (see [GH-28]).

### Fixed
- Correctly build macOS and Linux binaries on Travis CI.

[GH-28]: https://github.com/lunaryorn/mdcat/issues/28

## [0.9.0] – 2018-08-26
### Added
- `mdcat` builds on Windows now (see [GH-33] and [GH-34]).

### Changed
- Refactor internal terminal representation, replacing the terminal enum with a
  new `Terminal` trait and dynamic dispatch (see [GH-35]).
- Allow to disable specific terminal backends (see [GH-35]).
- Update minimum Rust version to 1.27.

[GH-33]: https://github.com/lunaryorn/mdcat/pull/33
[GH-34]: https://github.com/lunaryorn/mdcat/pull/34
[GH-35]: https://github.com/lunaryorn/mdcat/pull/35

## [0.8.0] – 2018-02-15
### Added
- Render SVG images in iTerm2 with `rsvg-convert` (requires `librsvg`).
- Expose `TerminalWrite` in `mdcat` crate (see [GH-20] by [@Byron]).

[GH-20]: https://github.com/lunaryorn/mdcat/pull/20
[@Byron]: https://github.com/Byron

## [0.7.0] – 2018-02-08
### Added
- Show images from HTTP and HTTPS URLs inline in iTerm2.
- Add `--local` flag to render only local images inline; for remote images, eg,
  HTTP URLs, show only the image title and the URL.
- Expose `mdcat` as a library crate (see [GH-18]), but with no guarantees
  about a stable interface, as `mdcat` stays at version 0.x for now.
- Show `--help` with colours.

### Changed
- Adhere to Semantic Versioning, but stay pre-1.0 so anything still goes.

[GH-18]: https://github.com/lunaryorn/mdcat/issues/18

## [0.6.0] – 2018-02-02
### Added
- Show inline images in [Terminology] (see [GH-16] by [@vinipsmaker]).

[Terminology]: http://terminolo.gy
[GH-16]: https://github.com/lunaryorn/mdcat/pull/16
[@vinipsmaker]: https://github.com/lunaryorn/vinipsmaker

### Changed
- Improve `--help` output: Hide some redundant options, add a bug reporting URL
  and explain the purpose of `mdcat`.
- Reduce dependencies and thus build time

## [0.5.0] – 2018-01-27
### Added
- Show links inline in iTerm2 and terminals based on VTE 0.50 or newer (see
  [GH-8], [GH-14] and [GH-15]).

[GH-8]: https://github.com/lunaryorn/mdcat/issues/8
[GH-14]: https://github.com/lunaryorn/mdcat/issues/14
[GH-15]: https://github.com/lunaryorn/mdcat/issues/15

### Changed
- Improve `--help` output.

### Fixed
- Remove redundant default value from `--colour` help text (see [GH-10], by [@wezm]).
- Replace light black with green; the former doesn't work with Solarized Dark.

[GH-10]: https://github.com/lunaryorn/mdcat/pull/10
[@wezm]: https://github.com/wezm

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

## [0.1.0] – 2018-01-14
### Added

- Support inline formatting.
- Support headings.
- Support code blocks and block quotes.
- Support ordered and unordered lists, with nest.
- Show links, with references grouped by section.

[0.1.0]: https://github.com/lunaryorn/mdcat/releases/tag/mdless-0.1.0
[0.1.1]: https://github.com/lunaryorn/mdcat/compare/mdless-0.1.0...mdless-0.1.1
[0.2.0]: https://github.com/lunaryorn/mdcat/compare/mdless-0.1.1...mdless-0.2.0
[0.3.0]: https://github.com/lunaryorn/mdcat/compare/mdless-0.2.0...mdcat-0.3.0
[0.4.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.3.0...mdcat-0.4.0
[0.5.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.4.0...mdcat-0.5.0
[0.6.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.5.0...mdcat-0.6.0
[0.7.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.6.0...mdcat-0.7.0
[0.8.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.7.0...mdcat-0.8.0
[0.9.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.8.0...mdcat-0.9.0
[0.9.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.9.0...mdcat-0.9.1
[0.9.2]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.9.1...mdcat-0.9.2
[0.10.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.9.2...mdcat-0.10.0
[0.10.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.10.0...mdcat-0.10.1
[0.11.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.10.1...mdcat-0.11.0
[0.12.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.11.0...mdcat-0.12.0
[0.12.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.12.0...mdcat-0.12.1
[0.13.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.12.1...mdcat-0.13.0
[0.14.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.13.0...mdcat-0.14.0
[0.15.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.14.0...mdcat-0.15.0
[0.15.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.15.0...mdcat-0.15.1
[0.16.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.15.1...mdcat-0.16.0
[0.16.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.16.0...mdcat-0.16.1
[0.17.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.16.1...mdcat-0.17.0
[0.17.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.17.0...mdcat-0.17.1
[0.18.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.17.1...mdcat-0.18.0
[0.18.1]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.18.0...mdcat-0.18.1
[0.18.2]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.18.1...mdcat-0.18.2
[0.18.3]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.18.2...mdcat-0.18.3
[0.18.4]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.18.3...mdcat-0.18.4
[0.19.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.18.4...mdcat-0.19.0
[0.20.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.19.0...mdcat-0.20.0
[0.21.0]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.20.0...mdcat-0.21.0
[Unreleased]: https://github.com/lunaryorn/mdcat/compare/mdcat-0.21.0...HEAD
