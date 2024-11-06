# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

Use `cargo release` to create a new release.

## [Unreleased]

### Removed
- Remove a few dependencies: `mime_guess` and `system_proxy` (see [GH-297]).
- mdcat no longer supports building against the `onig` regex library (see [GH-297]).
- mdcat no longer supports a static build against musl (see [GH-297]).

[GH-297]: https://github.com/swsnr/mdcat/pull/297

## [2.5.0] – 2024-09-26

### Changed
- Update to pulldown-cmark 0.12 (see [GH-276]).
  This notably improves whitespace handling for HTML blocks and inline HTML.
- mdcat now treats inline HTML like inline code, just with different styling (see [GH-276]).
  Specifically, mdcat now wraps inline HTML.

[GH-276]: https://github.com/swsnr/mdcat/pull/276

## [2.4.0] – 2024-09-20

### Added
- Add limited support for tables (see [GH-290]).
  Inline markup is stripped from table cells, and text is not wrapped.

### Changed
- Update dependencies.

[GH-290]: https://github.com/swsnr/mdcat/pull/290

## [2.3.1] – 2024-08-04

### Changed
- Release builds no longer use fat LTO, which significantly reduces compilation time.
- Update dependencies.

### Fixed
- mdcat no longer panics on markups in descriptions of rendered images (see [GH-287]).

[GH-287]: https://github.com/swsnr/mdcat/pull/287

## [2.3.0] – 2024-07-28

### Added
- mdcat now includes a `--completions` argument to generate shell completions for all major shells (see [GH-286]).

### Changed
- Update dependencies.

### Removed
- mdcat now longer builds shell completions and man pages during compilation (see [GH-286]).
  - Packagers now need to build the manpage explicitly during packaging.
- Release artifacts no longer contain completions; use `mdcat --completions` to generate them instead (see [GH-286]).

### Fixed
- Restore binary release artifacts (see [GH-284])

[GH-284]: https://github.com/swsnr/mdcat/issues/284
[GH-286]: https://github.com/swsnr/mdcat/pull/286

## [2.2.0] – 2024-07-11

### Changed

- Update a few vulnerable dependencies.
- Update resvg, which among other things enables colored emoji in SVG files (see [GH-283]).

[Gh-283]: https://github.com/swsnr/mdcat/pull/283

## [2.1.2] – 2024-03-11

### Changed

- Update dependencies to address Rust security advisories.

## [2.1.1] – 2024-01-15

### Changed

- Update all dependencies (see [GH-274]).

[GH-274]: https://github.com/swsnr/mdcat/pull/274

## [2.1.0] – 2023-10-16

### Added

- Support images in VSCode integrated terminal, 1.80 or newer (see [GH-266]).

### Changed
- When rendering iTerm2 images append `.png` to the file name reported to the terminal if mdcat rendered an SVG to PNG (see [GH-267]).
  Previously, mdcat retained the original file extension, and would ask iTerm2 to download a PNG image to an `.svg` file.

### Fixed
- Correct some iTerm2 inline image commands to better comply to the specification (see [GH-267]).
- Always terminate OSC commands with ST instead of BEL, as the latter is the legacy form (see [GH-267]).

[GH-266]: https://github.com/swsnr/mdcat/pull/266
[GH-267]: https://github.com/swsnr/mdcat/pull/267

## [2.0.4] – 2023-10-03

### Changed
- Update all dependencies.
- Bump MSRV to 1.72.

## [2.0.3] – 2023-04-24

### Changed
- mdcat now uses the kitty protocol to render images on WezTerm (see [GH-258]).
- mdcat now downscales images to the column limit if rendering with the kitty protocol (see [GH-258]).
    Previously mdcat scaled down to the window size, which looked strange if a given `--columns` was much smaller than the window size.

[GH-258]: https://github.com/swsnr/mdcat/pull/258

## [2.0.2] – 2023-04-19

### Changed

- Update dependencies.

### Fixed

- Fix SVG rendering:
    - Correctly enable SVG rendering and image processing features by default in `mdcat` (see [GH-256]).
    - Ignore `charset` and other mime type parameters when checking for `image/svg+xml` (see [GH-256]).

[GH-256]: https://github.com/swsnr/mdcat/pull/256

## [2.0.1] – 2023-04-16

### Fixed
- Properly reset line wrapping state in list items (see [GH-254]).
- Flush trailing spaces before starting a link to avoid link styling over an initial whitespace (see [GH-255]).

[GH-254]: https://github.com/swsnr/mdcat/pull/254
[GH-255]: https://github.com/swsnr/mdcat/pull/255

## [2.0.0] – 2023-04-15

### Added
- mdcat now fills paragraph text to the column limit, i.e. fills up short lines and wraps long lines (see [GH-4]).
- mdcat now allows to control color and style via a new `theme` field in `pulldown_cmark_mdcat::Settings` of type `pulldown_cmark_mdcat::Theme` (see [GH-48]).
    `pulldown_cmark_mdcat::Theme::default()` provides the standard mdcat 1.x colors and style.
- mdcat now exposes resource handling via the new `pulldown_cmark_mdcat::resources::ResourceUrlHandler` trait (see [GH-247]).
- `pulldown_cmark_mdcat` allows to disable SVG support and thus avoid the `resvg` dependency by disabling the `svg` feature (see [GH-249]).
- `pulldown_cmark_mdcat` allows to disable image processing support and thus avoid the `image` dependency by disabling the `image-processing` feature (see [GH-250]).

### Changed
- Update all dependencies.
- `mdcat::Settings` now holds a reference to a syntax set, so the syntax set can now be shared among multiple different settings.
- Explicitly set minimum rust version in `Cargo.toml`, and document MSRV policy.
- Move all core rendering functions into a new crate `pulldown-cmark-mdcat`; `mdcat` itself only contains the argument parsing and handling now (see [GH-248]).
    If you were using `mdcat` as a library before, you likely want to use `pulldown-cmark-mdcat` now.
- Move HTTP resource handling into new crate `mdcat-http-reqwest`, in order to isolate the rather heavy `reqwest` dependency (see [GH-248]).
- Increase timeouts for HTTP resources to avoid aborting too early.

### Removed
- `mdcat::Settings.resource_access` and the corresponding `ResourceAccess` enum (see [GH-247]).

[GH-4]: https://github.com/swsnr/mdcat/issues/4
[GH-48]: https://github.com/swsnr/mdcat/issues/48
[GH-247]: https://github.com/swsnr/mdcat/pull/247
[GH-248]: https://github.com/swsnr/mdcat/pull/248
[GH-249]: https://github.com/swsnr/mdcat/pull/249
[GH-250]: https://github.com/swsnr/mdcat/pull/250

## [1.1.1] – 2023-03-18

### Fixed
- No longer elide tracing info below warn level in release builds (see [GH-242]).
  This allows downstream consumers to keep tracing info included in their release builds.

[GH-242]: https://github.com/swsnr/mdcat/issues/242

## [1.1.0] – 2023-02-27

### Changed
- Update all dependencies.
  This removes a transitive dependency on a vulnerable version of `remove_dir_all`, see [GHSA-mc8h-8q98-g5hr].
- No longer sniff mime type from contents to identify SVG images.
  Instead rely on the `Content-Type` header for HTTP(S) images and the file extension for local resources (see [GH-239]).
- Render SVG images using the pure Rust `resvg` crate instead of `rsvg-convert`; mdcat no longer requires the latter tool at runtime (see [GH-240]).

### Fixed
- Use `less -r` instead of `less -R` in `mdless` if both `$PAGER` and `$MDCAT_PAGER` are unset (see [GH-238]).
- Time out external resources if no data was read for 100ms. Previously mdcat waited for 1s before timing out (see [GH-241]).

[GH-236]: https://github.com/swsnr/mdcat/pull/236
[GH-238]: https://github.com/swsnr/mdcat/issues/238
[GH-239]: https://github.com/swsnr/mdcat/pull/239
[GH-240]: https://github.com/swsnr/mdcat/pull/240
[GH-241]: https://github.com/swsnr/mdcat/pull/241

[GHSA-mc8h-8q98-g5hr]: https://github.com/advisories/GHSA-mc8h-8q98-g5hr

## [1.0.0] – 2023-01-07

### Added
- Add `--detect-terminal` to print the name of the detected terminal program (see [GH-232]).
- Add `--ansi` to skip terminal detection and use ANSI-formatting only (see [GH-232]).

### Changed

- Replace `ureq` with `reqwest` (see [GH-229]).
    This implies that the default build now creates a binary linked against the system standard SSL library, i.e. openssl under Linux.
    A fully static build now requires `--no-default-features --features static` for `cargo build`.
- Terminal detection always checks `$TERM` first and trusts its value if it denotes a specific terminal emulator (see [GH-232]).
- Update all dependencies.

### Fixed

- Correctly detect kitty started from iTerm (see [GH-230] and [GH-232]).

[GH-229]: https://github.com/swsnr/mdcat/pull/229
[GH-230]: https://github.com/swsnr/mdcat/pull/230
[GH-232]: https://github.com/swsnr/mdcat/pull/232

## [0.30.3] – 2022-12-01

### Fixed
- Fix release workflow to restore release artifacts (see [GH-218]).

[GH-218]: https://github.com/swsnr/mdcat/pull/218

## [0.30.2] – 2022-12-01

### Changed
- Update Github URL to <https://github.com/swsnr/mdcat>.
- Update dependencies.

## [0.30.1] – 2022-11-29

### Fixed
- Fix workflow syntax error to restore release artifacts.

## [0.30.0] – 2022-11-29

### Added
- Generate completions for `mdless` (see [GH-216]).

### Fixed
- Include generated shell completions in release artifacts.
- Fix completions for mdcat (see [GH-214] and [GH-216])

[GH-214]: https://github.com/swsnr/mdcat/issues/214
[GH-216]: https://github.com/swsnr/mdcat/pull/216

## [0.29.0] – 2022-10-21

### Changed
- Move repository back to <https://github.com/swsnr/mdcat>.
- Restore release binaries.
- Update dependencies, in particular clap to 4.0.15.

### Removed
- Support for `tree_magic_mini` for mime-type detection; mdcat now only uses the `file` tool (see [GH-204]).

[GH-204]: https://github.com/swsnr/mdcat/pull/204

## [0.28.0] – 2022-07-31

### Changed
- Update all dependencies, in particular syntect to 5.0.0 and pulldown-cmark to 0.9.2.

## [0.27.1] – 2022-04-17

### Fixed
- Build error on Windows (see [#201]).

[#201]: https://codeberg.org/flausch/mdcat/pulls/201

## [0.27.0] – 2022-04-10

### Added
- Add extensive tracing output, to aid debugging (see [#147]).

### Changed
- mdcat no longer invokes `file` to detect SVG images, but now requires the presence of a system-wide magic database (see [#154]).
  Disable default features to restore the previous behaviour to invoke `file` to detect mimetypes.

### Fixed
- File completion with zsh (see [#198]).

[#147]: https://codeberg.org/flausch/mdcat/issues/147
[#154]: https://codeberg.org/flausch/mdcat/issues/154
[#198]: https://codeberg.org/flausch/mdcat/issues/198

## [0.26.1] – 2022-02-12

### Changed
- Moved repository to <https://codeberg.org/flausch/mdcat>.

## [0.26.0] – 2022-02-12

### Changed
- Always output links as OSC-8 hyperlinks unless `--dump` is given.
  In particular, mdcat now prints hyperlinks if invoked as `mdless` or with `-p`, as recent `less` versions support OCS-8 hyperlinks (see [#191]).

[#191]: https://github.com/swsnr/mdcat/issues/191

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

[#192]: https://github.com/swsnr/mdcat/pull/192
[#193]: https://github.com/swsnr/mdcat/pull/193

## [0.24.1] – 2021-10-30

### Fixed
- Fix semi-broken release.
- Document support for foot in manpage.

## [0.24.0] – 2021-10-30

### Added
- Support for [foot](https://codeberg.org/dnkl/foot/) (see [#190]).

[#190]: https://github.com/swsnr/mdcat/pull/190

## [0.23.2] – 2021-07-18

### Changed
- WezTerm is now detected by `TERM` and `TERM_PROGRAM` environment variables (see [#186]).

[#186]: https://github.com/swsnr/mdcat/pull/186

## [0.23.1] – 2021-07-14

### Changed
- Use `TERM_PROGRAM` for determining WezTerm terminal (see [#185]).

[#185]: https://github.com/swsnr/mdcat/pull/185

## [0.23.0] – 2021-07-04

### Added
- Support for [WezTerm](https://wezfurlong.org/wezterm/) (see [#182]).
- Add PowerShell completions (see [#183] and [#184]).

[#182]: https://github.com/swsnr/mdcat/pull/182
[#183]: https://github.com/swsnr/mdcat/issues/183
[#184]: https://github.com/swsnr/mdcat/pull/184

## [0.22.4] – 2021-04-15

### Changed
- Update dependencies

## [0.22.3] – 2021-02-22
### Added
- Refuse to read more than 100MiB from external resources, e.g. images; mdcat cannot display images of that size reasonably anyway (see [#176]).

### Fixed
- Fix type error on FreeBSD (see [#177]).

[#176]: https://github.com/swsnr/mdcat/pull/176
[#177]: https://github.com/swsnr/mdcat/issues/177

## [0.22.2] – 2021-01-01

### Changed
- Replace `reqwest` with `ureq` to fetch images via HTTP/HTTPS (see [#168] and [#169]);
    the latter has considerably less dependencies and builds faster.
    It also builds statically out of the box, hence the static musl builds no longer require `curl` to fetch images.

### Removed
- The `reqwest` cargo feature (see [#168] and [#169]).

[#168]: https://github.com/swsnr/mdcat/issues/168
[#169]: https://github.com/swsnr/mdcat/pull/169

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
[#165]: https://github.com/swsnr/mdcat/pull/165
[#166]: https://github.com/swsnr/mdcat/pull/166
[#167]: https://github.com/swsnr/mdcat/pull/167

## [0.21.1] – 2020-09-01

### Fixed
- Update pulldown cmark to correctly ignore footnote refs (see [#155]).

[#155]: https://github.com/swsnr/mdcat/issues/155

## [0.21.0] – 2020-08-12

### Added
- Add `--paginate` flag to paginate the output of mdcat (see [#152]);
  with this flag mdcat sends it output to `less -R` or any alternative pager set in `$MDCAT_PAGER` or `$PAGER`.
  This flag also disables all terminal-specific formatting as the pager likely won't support it.
- Paginate output by default if invoked as `mdless`, that is, if `mdcat` is hard-linked to `mdless`.

### Fixed
- Render email autolinks (i.e. `<hello@example.com>`) as `mailto:` links.

[#152]: https://github.com/swsnr/mdcat/issues/152

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

[#140]: https://github.com/swsnr/mdcat/issues/140
[#141]: https://github.com/swsnr/mdcat/issues/141
[#149]: https://github.com/swsnr/mdcat/issues/149
[Inline Images Protocol]: https://iterm2.com/documentation-images.html

## [0.19.0] – 2020-06-19
### Added
- Release packages now include generated shell completions for Bash, Zsh and Fish.

### Changed
- Blockquotes no longer have green foreground text (see [#144]).

[#144]: https://github.com/swsnr/mdcat/issues/144

## [0.18.4] – 2020-06-14
### Fixed
- Fix typo in release workflow.
- Update all dependencies to no longer depend on yanked crate versions.

## [0.18.3] – 2020-06-14
### Fixed
- Properly ignore alt text of inline images (see [#148]).

[#148]: https://github.com/swsnr/mdcat/issues/148

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

[#142]: https://github.com/swsnr/mdcat/issues/142
[#139]: https://github.com/swsnr/mdcat/issues/139
[anyhow]: https://docs.rs/crate/anyhow
[#146]: https://github.com/swsnr/mdcat/issues/146

## [0.17.1] – 2020-05-24
### Fixed
- Correctly scale down large images on [kitty] (see [#124] and [#133] by
  [@fspillner]).

[#124]: https://github.com/swsnr/mdcat/issues/124
[#133]: https://github.com/swsnr/mdcat/pull/133

## [0.17.0] – 2020-05-20
### Changed
- `mdcat` is now distributed under the [MPL 2](http://mozilla.org/MPL/2.0/) license;
  some source files remain Apache 2.0 due to 3rd party rights (see [#138]).

### Fixed
- Do not fail with broken pipe error when rending large images (see [#134] by
  [@fspillner]).

[#134]: https://github.com/swsnr/mdcat/issues/134
[#138]: https://github.com/swsnr/mdcat/issues/138

## [0.16.1] – 2020-05-15
### Changed
- `mdcat::push_tty` now takes a `mdcat::Settings` struct which groups all
  external settings.

### Fixed
- Ignore broken pipes; `mdcat | head` no longer errors when `head` closes stdout
  of mdcat early (see [#136]).

[#136]: https://github.com/swsnr/mdcat/issues/136

## [0.16.0] – 2020-04-11
### Changed
- Upgrade to syntect 4.1 and enable its pure Rust regex backend to simplify
  building (see [#131]).  This crate now builds without Clang which fixes
  Clang-related build issues (see [#90])

[#90]: https://github.com/swsnr/mdcat/issues/90
[#131]: https://github.com/swsnr/mdcat/pull/131

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

[#114]: https://github.com/swsnr/mdcat/pull/114
[#115]: https://github.com/swsnr/mdcat/pull/115
[#54]: https://github.com/swsnr/mdcat/issues/54
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
[#65]: https://github.com/swsnr/mdcat/issues/65
[#84]: https://github.com/swsnr/mdcat/issues/84
[#104]: https://github.com/swsnr/mdcat/pull/104
[@fspillner]: https://github.com/fspillner

## [0.13.0] – 2019-03-30
### Added
- Render task lists nicely (see [#72]).
- Render strike-through text (see [#71]).  Some terminals do not support this
  feature, and mdcat does not have a fallback currently (see [#73]).

[#73]: https://github.com/swsnr/mdcat/issues/73
[#72]: https://github.com/swsnr/mdcat/issues/72
[#71]: https://github.com/swsnr/mdcat/issues/71

## [0.12.1] – 2018-12-24
### Fixed
- Do not add newline after inline text with styles disabled (see [#49]).

[#49]: https://github.com/swsnr/mdcat/issues/49

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
[#42]: https://github.com/swsnr/mdcat/pull/42
[#44]: https://github.com/swsnr/mdcat/pull/44
[#45]: https://github.com/swsnr/mdcat/pull/45
[#46]: https://github.com/swsnr/mdcat/issues/46

## [0.11.0] – 2018-10-25
### Changed
- Always print colours regardless of whether stdout if a tty or not.
- Replace `--colour` option with a `--no-colour` flag to turn off styled output.
- `mdcat::push_tty` no longer takes ownership of the `terminal` argument (see
  [#41]).
- Travis CI builds Windows binaries now.
- Test formatting output.

[#41]: https://github.com/swsnr/mdcat/issues/41

## [0.10.1] – 2018-09-09
### Fixed
- Properly package musl binary on Travis CI; restores Linux binary in releases.

## [0.10.0] – 2018-09-09
### Added
- Support colours on Windows 10 console (see [#36]).
- Support musl target on Linux (see [#37] and [#38]).
- Published Linux binary statically links musl now, and has no runtime
  dependencies (see [#29] and [#38]).

[#29]: https://github.com/swsnr/mdcat/issues/29
[#36]: https://github.com/swsnr/mdcat/pull/36
[#37]: https://github.com/swsnr/mdcat/issues/37
[#38]: https://github.com/swsnr/mdcat/pull/38

## [0.9.2] – 2018-08-26
### Fixed
- Do not falsely ignore deployments from Travis CI for Linux and macOS.

## [0.9.1] – 2018-08-26
### Added
- Publish binaries for Linux, macOS and Windows (see [#28]).

### Fixed
- Correctly build macOS and Linux binaries on Travis CI.

[#28]: https://github.com/swsnr/mdcat/issues/28

## [0.9.0] – 2018-08-26
### Added
- `mdcat` builds on Windows now (see [#33] and [#34]).

### Changed
- Refactor internal terminal representation, replacing the terminal enum with a
  new `Terminal` trait and dynamic dispatch (see [#35]).
- Allow to disable specific terminal backends (see [#35]).
- Update minimum Rust version to 1.27.

[#33]: https://github.com/swsnr/mdcat/pull/33
[#34]: https://github.com/swsnr/mdcat/pull/34
[#35]: https://github.com/swsnr/mdcat/pull/35

## [0.8.0] – 2018-02-15
### Added
- Render SVG images in iTerm2 with `rsvg-convert` (requires `librsvg`).
- Expose `TerminalWrite` in `mdcat` crate (see [#20] by [@Byron]).

[#20]: https://github.com/swsnr/mdcat/pull/20
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

[#18]: https://github.com/swsnr/mdcat/issues/18

## [0.6.0] – 2018-02-02
### Added
- Show inline images in [Terminology] (see [#16] by [@vinipsmaker]).

[Terminology]: http://terminolo.gy
[#16]: https://github.com/swsnr/mdcat/pull/16
[@vinipsmaker]: https://github.com/vinipsmaker

### Changed
- Improve `--help` output: Hide some redundant options, add a bug reporting URL
  and explain the purpose of `mdcat`.
- Reduce dependencies and thus build time

## [0.5.0] – 2018-01-27
### Added
- Show links inline in iTerm2 and terminals based on VTE 0.50 or newer (see
  [#8], [#14] and [#15]).

[#8]: https://github.com/swsnr/mdcat/issues/8
[#14]: https://github.com/swsnr/mdcat/issues/14
[#15]: https://github.com/swsnr/mdcat/issues/15

### Changed
- Improve `--help` output.

### Fixed
- Remove redundant default value from `--colour` help text (see [#10], by [@wezm]).
- Replace light black with green; the former doesn't work with Solarized Dark.

[#10]: https://github.com/swsnr/mdcat/pull/10
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

[Unreleased]: https://github.com/swsnr/mdcat/compare/mdcat-2.5.0...HEAD
[2.5.0]: https://github.com/swsnr/mdcat/compare/mdcat-2.4.0...mdcat-2.5.0
[2.4.0]: https://github.com/swsnr/mdcat/compare/mdcat-2.3.1...mdcat-2.4.0
[2.3.1]: https://github.com/swsnr/mdcat/compare/mdcat-2.3.0...mdcat-2.3.1
[2.3.0]: https://github.com/swsnr/mdcat/compare/mdcat-2.2.0...mdcat-2.3.0
[2.2.0]: https://github.com/swsnr/mdcat/compare/mdcat-2.1.2...mdcat-2.2.0
[2.1.2]: https://github.com/swsnr/mdcat/compare/mdcat-2.1.1...mdcat-2.1.2
[2.1.1]: https://github.com/swsnr/mdcat/compare/mdcat-2.1.0...mdcat-2.1.1
[2.1.0]: https://github.com/swsnr/mdcat/compare/mdcat-2.0.4...mdcat-2.1.0
[2.0.4]: https://github.com/swsnr/mdcat/compare/mdcat-2.0.3...mdcat-2.0.4
[2.0.3]: https://github.com/swsnr/mdcat/compare/mdcat-2.0.2...mdcat-2.0.3
[2.0.2]: https://github.com/swsnr/mdcat/compare/mdcat-2.0.1...mdcat-2.0.2
[2.0.1]: https://github.com/swsnr/mdcat/compare/mdcat-2.0.0...mdcat-2.0.1
[2.0.0]: https://github.com/swsnr/mdcat/compare/mdcat-1.1.1...mdcat-2.0.0
[1.1.1]: https://github.com/swsnr/mdcat/compare/mdcat-1.1.0...mdcat-1.1.1
[1.1.0]: https://github.com/swsnr/mdcat/compare/mdcat-1.0.0...mdcat-1.1.0
[1.0.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.30.3...mdcat-1.0.0
[0.30.3]: https://github.com/swsnr/mdcat/compare/mdcat-0.30.2...mdcat-0.30.3
[0.30.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.30.1...mdcat-0.30.2
[0.30.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.30.0...mdcat-0.30.1
[0.30.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.29.0...mdcat-0.30.0
[0.29.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.28.0...mdcat-0.29.0
[0.28.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.27.1...mdcat-0.28.0
[0.27.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.27.0...mdcat-0.27.1
[0.27.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.26.1...mdcat-0.27.0
[0.26.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.26.0...mdcat-0.26.1
[0.26.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.25.1...mdcat-0.26.0
[0.25.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.25.0...mdcat-0.25.1
[0.25.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.24.2...mdcat-0.25.0
[0.24.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.24.1...mdcat-0.24.2
[0.24.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.24.0...mdcat-0.24.1
[0.24.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.23.2...mdcat-0.24.0
[0.23.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.23.1...mdcat-0.23.2
[0.23.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.23.0...mdcat-0.23.1
[0.23.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.22.4...mdcat-0.23.0
[0.22.4]: https://github.com/swsnr/mdcat/compare/mdcat-0.22.3...mdcat-0.22.4
[0.22.3]: https://github.com/swsnr/mdcat/compare/mdcat-0.22.2...mdcat-0.22.3
[0.22.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.22.1...mdcat-0.22.2
[0.22.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.22.0...mdcat-0.22.1
[0.22.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.21.1...mdcat-0.22.0
[0.21.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.21.0...mdcat-0.21.1
[0.21.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.20.0...mdcat-0.21.0
[0.20.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.19.0...mdcat-0.20.0
[0.19.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.18.4...mdcat-0.19.0
[0.18.4]: https://github.com/swsnr/mdcat/compare/mdcat-0.18.3...mdcat-0.18.4
[0.18.3]: https://github.com/swsnr/mdcat/compare/mdcat-0.18.2...mdcat-0.18.3
[0.18.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.18.1...mdcat-0.18.2
[0.18.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.18.0...mdcat-0.18.1
[0.18.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.17.1...mdcat-0.18.0
[0.17.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.17.0...mdcat-0.17.1
[0.17.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.16.1...mdcat-0.17.0
[0.16.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.16.0...mdcat-0.16.1
[0.16.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.15.1...mdcat-0.16.0
[0.15.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.15.0...mdcat-0.15.1
[0.15.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.14.0...mdcat-0.15.0
[0.14.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.13.0...mdcat-0.14.0
[0.13.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.12.1...mdcat-0.13.0
[0.12.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.12.0...mdcat-0.12.1
[0.12.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.11.0...mdcat-0.12.0
[0.11.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.10.1...mdcat-0.11.0
[0.10.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.10.0...mdcat-0.10.1
[0.10.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.9.2...mdcat-0.10.0
[0.9.2]: https://github.com/swsnr/mdcat/compare/mdcat-0.9.1...mdcat-0.9.2
[0.9.1]: https://github.com/swsnr/mdcat/compare/mdcat-0.9.0...mdcat-0.9.1
[0.9.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.8.0...mdcat-0.9.0
[0.8.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.7.0...mdcat-0.8.0
[0.7.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.6.0...mdcat-0.7.0
[0.6.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.5.0...mdcat-0.6.0
[0.5.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.4.0...mdcat-0.5.0
[0.4.0]: https://github.com/swsnr/mdcat/compare/mdcat-0.3.0...mdcat-0.4.0
[0.3.0]: https://github.com/swsnr/mdcat/compare/mdless-0.2.0...mdcat-0.3.0
[0.2.0]: https://github.com/swsnr/mdcat/compare/mdless-0.1.1...mdless-0.2.0
[0.1.1]: https://github.com/swsnr/mdcat/compare/mdless-0.1.0...mdless-0.1.1
[0.1.0]: https://github.com/swsnr/mdcat/releases/tag/mdless-0.1.0
