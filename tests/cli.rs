// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test the command line interface of mdcat

#![deny(warnings, missing_docs, clippy::all)]

mod cli {
    use std::ffi::OsStr;
    use std::process::{Command, Output};

    fn run_cargo_mdcat<I, S>(args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("cargo")
            .arg("run")
            .arg("-q")
            .arg("--")
            .args(args)
            .output()
            .unwrap()
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
        assert!(stdout.contains("mdcat uses the standardized CommonMark dialect"));
    }

    #[test]
    fn file_list_fail_late() {
        let output = run_cargo_mdcat(&["does-not-exist", "sample/common-mark.md"]);
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(!output.status.success());
        // We failed to read the first file but still printed the second.
        assert!(
            stderr.contains("Error: does-not-exist: No such file or directory (os error 2)"),
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
            stderr.contains("Error: does-not-exist: No such file or directory (os error 2)"),
            "Stderr: {}",
            stderr
        );
        assert!(output.stdout.is_empty());
    }
}
