// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use std::ffi::OsString;
use snafu::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct Git {
    args: Vec<OsString>,
}

impl Git {
    pub fn new() -> Self {
        todo!();
    }

    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        todo!();
    }

    pub fn run(&self) -> Result<(), GitError> {
        todo!();
    }
}

/// Git error type public API.
#[derive(Debug, Snafu)]
pub struct GitError(InnerGitError);

/// Alias to allow one-off functions with different error type.
pub type Result<T, E = GitError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerGitError {}

#[cfg(test)]
mod tests {
    use super::*;
}
