# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

To publish a new release run `scripts/release` from the project directory.

## [Unreleased]

### Changed
- Moved repository to <https://codeberg.org/flausch/mdcat>.

## [0.26.0] – 2022-02-12

### Changed
- Always output links as OSC-8 hyperlinks unless `--dump` is given.
  In particular, mdcat now prints hyperlinks if invoked as `mdless` or with `-p`, as recent `less` versions support OCS-8 hyperlinks (see [#191]).

[#191]: https://codeberg.org/flausch/mdcat/issues/191

### Removed
- mdcat no longer attempts to detect OSC8 link support of the underlying terminal.

## [0.25.1] – 2022-01-17

### Changed
- Update pulldown-cmark to 0.9.1 which fixes a minor parsing issue.

## [0.25.0] – 2021-12-23

### Changed
- Update pulldown-cmark to 0.9.
- Buffer writes to terminal and pager, to reduce the amount of syscalls.

## [0.24.2] – 2021-11-19

### Added
- Support for `$TERM=foot*` (see [#193]).

### Fixed
- Fix compiler error with newer `anyhow` versions (see [#192]).

[#192]: https://codeberg.org/flausch/mdcat/pulls/192
[#193]: https://codeberg.org/flausch/mdcat/pulls/193

## [0.24.1] – 2021-10-30

### Fixed
- Fix semi-broken release.
- Document support for foot in manpage.

## [0.24.0] – 2021-10-30

### Added
- Support for [foot](https://codeberg.org/dnkl/foot/) (see [#190]).

[#190]: https://codeberg.org/flausch/mdcat/pulls/190

## [0.23.2] – 2021-07-18

### Changed
- WezTerm is now detected by `TERM` and `TERM_PROGRAM` environment variables (see [#186]).

[#186]: https://codeberg.org/flausch/mdcat/pulls/186

## [0.23.1] – 2021-07-14

### Changed
- Use `TERM_PROGRAM` for determining WezTerm terminal (see [#185]).

[#185]: https://codeberg.org/flausch/mdcat/pulls/185

## [0.23.0] – 2021-07-04

### Added
- Support for [WezTerm](https://wezfurlong.org/wezterm/) (see [#182]).
- Add PowerShell completions (see [#183] and [#184]).

[#182]: https://codeberg.org/flausch/mdcat/pulls/182
[#183]: https://codeberg.org/flausch/mdcat/issues/183
[#184]: https://codeberg.org/flausch/mdcat/pulls/184

## [0.22.4] – 2021-04-15

### Changed
- Update dependencies

## [0.22.3] – 2021-02-22
### Added
- Refuse to read more than 100MiB from external resources, e.g. images; mdcat cannot display images of that size reasonably anyway (see [#176]).

### Fixed
- Fix type error on FreeBSD (see [#177]).

[#176]: https://codeberg.org/flausch/mdcat/pulls/176
[#177]: https://codeberg.org/flausch/mdcat/issues/177

## [0.22.2] – 2021-01-01

### Changed
- Replace `reqwest` with `ureq` to fetch images via HTTP/HTTPS (see [#168] and [#169]);
    the latter has considerably less dependencies and builds faster.
    It also builds statically out of the box, hence the static musl builds no longer require `curl` to fetch images.

### Removed
- The `reqwest` cargo feature (see [#168] and [#169]).

[#168]: https://codeberg.org/flausch/mdcat/issues/168
[#169]: https://codeberg.org/flausch/mdcat/pulls/169

## [0.22.1] – 2020-10-17

### Fixed

- Include manpage source in Windows packages.
    Currently the manpage doesn't build on Windows CI.

## [0.22.0] – 2020-10-17

### Added
- Enable OSC8 hyperlinks in Kitty (see [#165]).
    Kitty supports hyperlinks since [version 0.19][kitty-0.19], see [Kitty #68].
    Note that `mdcat` *unconditionally* prints hyperlinks if it detects a kitty terminal.
    It makes no attempt to detect whether the Kitty version is compatible or the [`allow_hyperlinks`] setting is enabled.
- `mdcat --version` (but not `mdcat -V`) now informs whether HTTP/HTTPS support is builtin or requires `curl`.
- mdcat now includes a manpage (see [#167]).

### Changed
- `mdcat` now asks the controlling terminal for the terminal size and thus correctly detects the terminal size even if standard input, standard output and standard error are all redirected (see [#166]).
- `mdcat` no longer requires `kitty icat` to detect the size of kitty windows (see [#166]).
    Consequently mdcat can now show images on Kitty terminals even over SSH.
- `mdcat --help` no longer uses colours, and always wraps at 80 characters now.

[kitty-0.19]: https://sw.kovidgoyal.net/kitty/changelog.html#id2
[kitty #68]: https://github.com/kovidgoyal/kitty/issues/68
[`allow_hyperlinks`]: https://sw.kovidgoyal.net/kitty/conf.html?highlight=hyperlinks#opt-kitty.allow_hyperlinks
[#165]: https://codeberg.org/flausch/mdcat/pulls/165
[#166]: https://codeberg.org/flausch/mdcat/pulls/166
[#167]: https://codeberg.org/flausch/mdcat/pulls/167

## [0.21.1] – 2020-09-01

### Fixed
- Update pulldown cmark to correctly ignore footnote refs (see [#155]).

[#155]: https://codeberg.org/flausch/mdcat/issues/155

## [0.21.0] – 2020-08-12

### Added
- Add `--paginate` flag to paginate the output of mdcat (see [#152]);
  with this flag mdcat sends it output to `less -R` or any alternative pager set in `$MDCAT_PAGER` or `$PAGER`.
  This flag also disables all terminal-specific formatting as the pager likely won't support it.
- Paginate output by default if invoked as `mdless`, that is, if `mdcat` is hard-linked to `mdless`.

### Fixed
- Render email autolinks (i.e. `<hello@example.com>`) as `mailto:` links.

[#152]: https://codeberg.org/flausch/mdcat/issues/152

## [0.20.0] – 2020-07-05
### Added
- `mdcat::TerminalCapabilities` now exposes constructors for specific terminal emulators.
- Render reference link definitions as inline links if possible (see [#149]).
    This mainly affects image links inside inline links which get rendered as reference links.
- `mdcat::Environment` now contains all environment information required to render properly, namely the local hostname and the base URL.

### Changed
- `mdcat::push_tty` now takes an `mdcat::Environment` instead of `base_dir`.
    `base_dir` is now part of `mdcat::Environment`.
- Image links now use purple foreground text (see [#140] and [#149]).
- Image links render as inline links if the terminal does not support inline images and the image is not inside another link (see [#141]).
- `mdcat::TerminalCapabilities` now uses `Option` to denote missing capabilities.

### Fixed
- Always treat links targets as URLs, never as paths.
- On ITerm2 only use the last segment of image URLs as filename for inline images (see [#149]).
    Previously mdcat used the full URL based on a misunderstanding of the [Inline Images Protocol].

[#140]: https://codeberg.org/flausch/mdcat/issues/140
[#141]: https://codeberg.org/flausch/mdcat/issues/141
[#149]: https://codeberg.org/flausch/mdcat/issues/149
[Inline Images Protocol]: https://iterm2.com/documentation-images.html

## [0.19.0] – 2020-06-19
### Added
- Release packages now include generated shell completions for Bash, Zsh and Fish.

### Changed
- Blockquotes no longer have green foreground text (see [#144]).

[#144]: https://codeberg.org/flausch/mdcat/issues/144

## [0.18.4] – 2020-06-14
### Fixed
- Fix typo in release workflow.
- Update all dependencies to no longer depend on yanked crate versions.

## [0.18.3] – 2020-06-14
### Fixed
- Properly ignore alt text of inline images (see [#148]).

[#148]: https://codeberg.org/flausch/mdcat/issues/148

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
- New simpler rendering algorithm (see #[142]) which solves numerous rendering
  issues (see below).
- Handle internal errors with [anyhow] to add more context to errors (see
  [#139]}.
- `mdcat::push_tty` only fails with `std::io::Error`: `mdcat` never visibly
  fails unless it can’t write output.

### Fixed
- Respect `--local-only` and resource access policy; this got lost in some
  refactoring (see [#146]).
- Consistent margins and newlines around paragraphs, HTML blocks and inside list
  items (see [#142]).
- Correctly indent nested code blocks in lists and block quotes (see [#142]).
- No longer print leading blank lines before lists (see [#142]).
- Correctly indent block quotes (see [#142]).
- Colorize the entire text of links (see [#142]).

[#142]: https://codeberg.org/flausch/mdcat/issues/142
[#139]: https://codeberg.org/flausch/mdcat/issues/139
[anyhow]: https://docs.rs/crate/anyhow
[#146]: https://codeberg.org/flausch/mdcat/issues/146

## [0.17.1] – 2020-05-24
### Fixed
- Correctly scale down large images on [kitty] (see [#124] and [#133] by
  [@fspillner]).

[#124]: https://codeberg.org/flausch/mdcat/issues/124
[#133]: https://codeberg.org/flausch/mdcat/pulls/133

## [0.17.0] – 2020-05-20
### Changed
- `mdcat` is now distributed under the [MPL 2](http://mozilla.org/MPL/2.0/) license;
  some source files remain Apache 2.0 due to 3rd party rights (see [#138]).

### Fixed
- Do not fail with broken pipe error when rending large images (see [#134] by
  [@fspillner]).

[#134]: https://codeberg.org/flausch/mdcat/issues/134
[#138]: https://codeberg.org/flausch/mdcat/issues/138

## [0.16.1] – 2020-05-15
### Changed
- `mdcat::push_tty` now takes a `mdcat::Settings` struct which groups all
  external settings.

### Fixed
- Ignore broken pipes; `mdcat | head` no longer errors when `head` closes stdout
  of mdcat early (see [#136]).

[#136]: https://codeberg.org/flausch/mdcat/issues/136

## [0.16.0] – 2020-04-11
### Changed
- Upgrade to syntect 4.1 and enable its pure Rust regex backend to simplify
  building (see [#131]).  This crate now builds without Clang which fixes
  Clang-related build issues (see [#90])

[#90]: https://codeberg.org/flausch/mdcat/issues/90
[#131]: https://codeberg.org/flausch/mdcat/pulls/131

## [0.15.1] – 2020-02-15
### Changed
- Update pulldown-cmark to 0.7.

## [0.15.0] – 2020-01-11
### Added
- Release builds now perform full link-time optimization to create a smaller
  binary.  We do recommend to also `strip` the `mdcat` binary.
- Render SVG images in [kitty] (see [#114]).
- Update to reqwest 0.10.
- Process file list as input (see [#54] and [#115], by [@norman-abramovitz]):
    - Add `--fail` flag to exit on the first error when processing a file list;
      the default behaviour is to continue with the next file in case of error.

### Changed
- Replace `remote_resources` feature with `reqwest` feature to use reqwest for
  retrieving remote resources, and fall back to the `curl` command if `reqwest`
  is disabled.

[#114]: https://codeberg.org/flausch/mdcat/pulls/114
[#115]: https://codeberg.org/flausch/mdcat/pulls/115
[#54]: https://codeberg.org/flausch/mdcat/issues/54
[@norman-abramovitz]: https://github.com/norman-abramovitz

## [0.14.0] – 2019-12-18
### Added
- Render images in [kitty] (see [#65] and [#104] by [@fspillner]).

### Changed
- Update pulldown-cmark to 0.6 which supports CommonMark 0.29 and improves
  parser speed and correctness.
- Enable SIMD in pulldown-cmark to squeeze out the last bit of performance.
- Remove all features except `remote_resources` to reduce build complexity.

### Removed
- No longer depend on `immeta`.

[kitty]: https://sw.kovidgoyal.net/kitty/
[#65]: https://codeberg.org/flausch/mdcat/issues/65
[#84]: https://codeberg.org/flausch/mdcat/issues/84
[#104]: https://codeberg.org/flausch/mdcat/pulls/104
[@fspillner]: https://github.com/fspillner

## [0.13.0] – 2019-03-30
### Added
- Render task lists nicely (see [#72]).
- Render strike-through text (see [#71]).  Some terminals do not support this
  feature, and mdcat does not have a fallback currently (see [#73]).

[#73]: https://codeberg.org/flausch/mdcat/issues/73
[#72]: https://codeberg.org/flausch/mdcat/issues/72
[#71]: https://codeberg.org/flausch/mdcat/issues/71

## [0.12.1] – 2018-12-24
### Fixed
- Do not add newline after inline text with styles disabled (see [#49]).

[#49]: https://codeberg.org/flausch/mdcat/issues/49

## [0.12.0] – 2018-12-20
### Added
- Add `TerminalCapability` struct as replacement for `mdcat::Terminal` trait to
  remove dynamic dispatch and allow for more accurate and less complicated
  conditional compilation of terminal support for different platforms (see
  [#45]).
- Move to Rust 2018, and raise minimum supported Rust version to 1.31 (see
  [#46]).

### Changed
- Drop support for Rust 1.29 and older.
- Do not test specific Rust on versions on Travis CI any longer; Rust stable
  becomes the lowest supported Rust version.

### Removed
- `mdcat::Terminal` trait and implementations (see [#45]).

### Fixed
- Set hostname to local hostname for inline links to `file://` URLs, which
  should properly resolve `file://` URLs over SSH (see [OSC 8 file URLs],
  [#42] and [#44]).

[OSC 8 file URLs]: https://git.io/vd4ee#file-uris-and-the-hostname
[#42]: https://codeberg.org/flausch/mdcat/pulls/42
[#44]: https://codeberg.org/flausch/mdcat/pulls/44
[#45]: https://codeberg.org/flausch/mdcat/pulls/45
[#46]: https://codeberg.org/flausch/mdcat/issues/46

## [0.11.0] – 2018-10-25
### Changed
- Always print colours regardless of whether stdout if a tty or not.
- Replace `--colour` option with a `--no-colour` flag to turn off styled output.
- `mdcat::push_tty` no longer takes ownership of the `terminal` argument (see
  [#41]).
- Travis CI builds Windows binaries now.
- Test formatting output.

[#41]: https://codeberg.org/flausch/mdcat/issues/41

## [0.10.1] – 2018-09-09
### Fixed
- Properly package musl binary on Travis CI; restores Linux binary in releases.

## [0.10.0] – 2018-09-09
### Added
- Support colours on Windows 10 console (see [#36]).
- Support musl target on Linux (see [#37] and [#38]).
- Published Linux binary statically links musl now, and has no runtime
  dependencies (see [#29] and [#38]).

[#29]: https://codeberg.org/flausch/mdcat/issues/29
[#36]: https://codeberg.org/flausch/mdcat/pulls/36
[#37]: https://codeberg.org/flausch/mdcat/issues/37
[#38]: https://codeberg.org/flausch/mdcat/pulls/38

## [0.9.2] – 2018-08-26
### Fixed
- Do not falsely ignore deployments from Travis CI for Linux and macOS.

## [0.9.1] – 2018-08-26
### Added
- Publish binaries for Linux, macOS and Windows (see [#28]).

### Fixed
- Correctly build macOS and Linux binaries on Travis CI.

[#28]: https://codeberg.org/flausch/mdcat/issues/28

## [0.9.0] – 2018-08-26
### Added
- `mdcat` builds on Windows now (see [#33] and [#34]).

### Changed
- Refactor internal terminal representation, replacing the terminal enum with a
  new `Terminal` trait and dynamic dispatch (see [#35]).
- Allow to disable specific terminal backends (see [#35]).
- Update minimum Rust version to 1.27.

[#33]: https://codeberg.org/flausch/mdcat/pulls/33
[#34]: https://codeberg.org/flausch/mdcat/pulls/34
[#35]: https://codeberg.org/flausch/mdcat/pulls/35

## [0.8.0] – 2018-02-15
### Added
- Render SVG images in iTerm2 with `rsvg-convert` (requires `librsvg`).
- Expose `TerminalWrite` in `mdcat` crate (see [#20] by [@Byron]).

[#20]: https://codeberg.org/flausch/mdcat/pulls/20
[@Byron]: https://github.com/Byron

## [0.7.0] – 2018-02-08
### Added
- Show images from HTTP and HTTPS URLs inline in iTerm2.
- Add `--local` flag to render only local images inline; for remote images, eg,
  HTTP URLs, show only the image title and the URL.
- Expose `mdcat` as a library crate (see [#18]), but with no guarantees
  about a stable interface, as `mdcat` stays at version 0.x for now.
- Show `--help` with colours.

### Changed
- Adhere to Semantic Versioning, but stay pre-1.0 so anything still goes.

[#18]: https://codeberg.org/flausch/mdcat/issues/18

## [0.6.0] – 2018-02-02
### Added
- Show inline images in [Terminology] (see [#16] by [@vinipsmaker]).

[Terminology]: http://terminolo.gy
[#16]: https://codeberg.org/flausch/mdcat/pulls/16
[@vinipsmaker]: https://codeberg.org/flausch/vinipsmaker

### Changed
- Improve `--help` output: Hide some redundant options, add a bug reporting URL
  and explain the purpose of `mdcat`.
- Reduce dependencies and thus build time

## [0.5.0] – 2018-01-27
### Added
- Show links inline in iTerm2 and terminals based on VTE 0.50 or newer (see
  [#8], [#14] and [#15]).

[#8]: https://codeberg.org/flausch/mdcat/issues/8
[#14]: https://codeberg.org/flausch/mdcat/issues/14
[#15]: https://codeberg.org/flausch/mdcat/issues/15

### Changed
- Improve `--help` output.

### Fixed
- Remove redundant default value from `--colour` help text (see [#10], by [@wezm]).
- Replace light black with green; the former doesn't work with Solarized Dark.

[#10]: https://codeberg.org/flausch/mdcat/pulls/10
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

[Unreleased]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.26.0...HEAD
[0.26.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.25.1...mdcat-0.26.0
[0.25.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.25.0...mdcat-0.25.1
[0.25.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.24.2...mdcat-0.25.0
[0.24.2]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.24.1...mdcat-0.24.2
[0.24.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.24.0...mdcat-0.24.1
[0.24.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.23.2...mdcat-0.24.0
[0.23.2]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.23.1...mdcat-0.23.2
[0.23.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.23.0...mdcat-0.23.1
[0.23.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.22.4...mdcat-0.23.0
[0.22.4]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.22.3...mdcat-0.22.4
[0.22.3]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.22.2...mdcat-0.22.3
[0.22.2]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.22.1...mdcat-0.22.2
[0.22.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.22.0...mdcat-0.22.1
[0.22.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.21.1...mdcat-0.22.0
[0.21.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.21.0...mdcat-0.21.1
[0.21.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.20.0...mdcat-0.21.0
[0.20.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.19.0...mdcat-0.20.0
[0.19.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.18.4...mdcat-0.19.0
[0.18.4]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.18.3...mdcat-0.18.4
[0.18.3]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.18.2...mdcat-0.18.3
[0.18.2]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.18.1...mdcat-0.18.2
[0.18.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.18.0...mdcat-0.18.1
[0.18.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.17.1...mdcat-0.18.0
[0.17.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.17.0...mdcat-0.17.1
[0.17.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.16.1...mdcat-0.17.0
[0.16.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.16.0...mdcat-0.16.1
[0.16.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.15.1...mdcat-0.16.0
[0.15.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.15.0...mdcat-0.15.1
[0.15.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.14.0...mdcat-0.15.0
[0.14.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.13.0...mdcat-0.14.0
[0.13.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.12.1...mdcat-0.13.0
[0.12.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.12.0...mdcat-0.12.1
[0.12.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.11.0...mdcat-0.12.0
[0.11.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.10.1...mdcat-0.11.0
[0.10.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.10.0...mdcat-0.10.1
[0.10.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.9.2...mdcat-0.10.0
[0.9.2]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.9.1...mdcat-0.9.2
[0.9.1]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.9.0...mdcat-0.9.1
[0.9.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.8.0...mdcat-0.9.0
[0.8.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.7.0...mdcat-0.8.0
[0.7.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.6.0...mdcat-0.7.0
[0.6.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.5.0...mdcat-0.6.0
[0.5.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.4.0...mdcat-0.5.0
[0.4.0]: https://codeberg.org/flausch/mdcat/compare/mdcat-0.3.0...mdcat-0.4.0
[0.3.0]: https://codeberg.org/flausch/mdcat/compare/mdless-0.2.0...mdcat-0.3.0
[0.2.0]: https://codeberg.org/flausch/mdcat/compare/mdless-0.1.1...mdless-0.2.0
[0.1.1]: https://codeberg.org/flausch/mdcat/compare/mdless-0.1.0...mdless-0.1.1
[0.1.0]: https://codeberg.org/flausch/mdcat/releases/tag/mdless-0.1.0