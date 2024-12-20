// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use std::{
    ffi::OsString,
    process::Command,
    io::Error as IoError,
};
use snafu::prelude::*;
use log::info;

/// Git binary handler.
///
/// Manages system calls to user's Git binary to help manage Git repository
/// data.
#[derive(Debug, Default, Clone)]
pub struct Git {
    args: Vec<OsString>,
}

impl Git {
    /// Construct new Git handler.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add arguments to pass to Git binary.
    pub fn with_args(&mut self, args: impl IntoIterator<Item = impl Into<OsString>>) {
        self.args.extend(args.into_iter().map(Into::into));
    }

    /// Call Git binary.
    ///
    /// Will pass given arguments to Git binary. Will log and return any output
    /// Git has written to stdout after calling it. Any arguments given will
    /// also be cleared for new arguments to be passed later on.
    ///
    /// # Errors
    ///
    /// Will fail if system call to Git binary fails, or Git binary itself fails
    /// to execute with given arguments.
    pub fn run(&mut self) -> Result<String, GitError> {
        let output = Command::new("git").args(&self.args).output().context(SyscallSnafu)?;
        if !output.status.success() {
            let msg = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
            return Err(GitError(InnerGitError::GitBin { msg }));
        }

        let msg = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
        info!("{msg}");
        self.args.clear();

        Ok(msg)
    }
}

/// Git error type public API.
#[derive(Debug, Snafu)]
pub struct GitError(InnerGitError);

/// Alias to allow one-off functions with different error type.
pub type Result<T, E = GitError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerGitError {
    #[snafu(display("Failed to make syscall to Git binary"))]
    Syscall { source: IoError },

    #[snafu(display("{msg}"))]
    GitBin { msg: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;
    use snafu::{report, Whatever};
    use pretty_assertions::assert_eq;

    #[rstest]
    #[report]
    fn git_run_return_str() -> Result<(), Whatever> {
        let mut git = Git::new();
        git.with_args(["ls-files", "--", "README.md"]);
        let result = git.run().with_whatever_context(|_| "Failed to run Git binary")?;
        let expect = "README.md\n".to_string();
        assert_eq!(result, expect);

        Ok(())
    }
}
