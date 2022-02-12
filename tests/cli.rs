// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Test the command line interface of mdcat

#![deny(warnings, missing_docs, clippy::all)]

mod cli {
    use std::ffi::OsStr;
    use std::io::Read;
    use std::process::{Command, Output, Stdio};

    fn cargo_mdcat() -> Command {
        Command::new(env!("CARGO_BIN_EXE_mdcat"))
    }

    fn run_cargo_mdcat<I, S>(args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        cargo_mdcat().args(args).output().unwrap()
    }

    #[test]
    fn show_help() {
        let output = run_cargo_mdcat(&["--help"]);
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(
            output.status.success(),
            "non-zero exit code: {:?}",
            output.status,
        );
        assert!(output.stderr.is_empty());
        assert!(stdout.contains("See 'man 1 mdcat' for more information."));
    }

    #[test]
    fn file_list_fail_late() {
        let output = run_cargo_mdcat(&["does-not-exist", "sample/common-mark.md"]);
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(!output.status.success());
        // We failed to read the first file but still printed the second.
        assert!(
            stderr.contains("Error: does-not-exist:") && stderr.contains("(os error 2)"),
            "Stderr: {}",
            stderr
        );
        assert!(stdout.contains("CommonMark sample document"));
    }

    #[test]
    fn file_list_fail_fast() {
        let output = run_cargo_mdcat(&["--fail", "does-not-exist", "sample/common-mark.md"]);
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(!output.status.success());
        // We failed to read the first file and exited early, so nothing was printed at all
        assert!(
            stderr.contains("Error: does-not-exist:") && stderr.contains("(os error 2)"),
            "Stderr: {}",
            stderr
        );
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn ignore_broken_pipe() {
        let mut child = cargo_mdcat()
            .arg("sample/common-mark.md")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stderr = Vec::new();
        drop(child.stdout.take());
        child
            .stderr
            .as_mut()
            .unwrap()
            .read_to_end(&mut stderr)
            .unwrap();

        assert!(child.wait().unwrap().success());

        use pretty_assertions::assert_eq;
        assert_eq!(String::from_utf8_lossy(&stderr), "")
    }
}
