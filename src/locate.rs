// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use directories::BaseDirs;
use log::trace;
use snafu::prelude::*;
use std::path::{Path, PathBuf};

#[cfg(test)]
use mockall::automock;

/// Representation of a locator handler.
///
/// Provides _expected_ paths to configuration data. Path validation is left to
/// the caller to figure out.
#[cfg_attr(test, automock)]
pub trait Locator {
    fn home_dir(&self) -> &Path;
    fn config_dir(&self) -> &Path;
    fn hooks_dir(&self) -> &Path;
    fn repos_dir(&self) -> &Path;
}

/// Locator type that uses XDG Base Directory specification.
#[derive(Debug, Clone)]
pub struct XdgLocator {
    layout: BaseDirs,
    config_dir: PathBuf,
    hooks_dir: PathBuf,
    repos_dir: PathBuf,
}

impl XdgLocator {
    /// Construct new XDG locator type.
    ///
    /// Will determine expected configuration data paths according to XDG
    /// Base Directory specification.
    ///
    /// # Errors
    ///
    /// Will fail if home directory cannot be determined.
    pub fn locate() -> Result<Self, LocateError> {
        trace!("Determine configuration paths");
        let layout = BaseDirs::new().context(NoWayHomeSnafu)?;
        let config_dir = layout.config_dir().join("dotfiles-ocd");
        let hooks_dir = config_dir.join("hooks");
        let repos_dir = layout.data_dir().join("dotfiles-ocd");
        Ok(Self { layout, config_dir, hooks_dir, repos_dir })
    }
}

impl Locator for XdgLocator {
    fn home_dir(&self) -> &Path {
        self.layout.home_dir()
    }

    fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    fn hooks_dir(&self) -> &Path {
        &self.hooks_dir
    }

    fn repos_dir(&self) -> &Path {
        &self.repos_dir
    }
}

/// Locator error type for public API.
#[derive(Debug, Snafu)]
pub struct LocateError(InnerLocateError);

/// Alias to allow one-off functions with different error type.
pub type Result<T, E = LocateError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerLocateError {
    #[snafu(display("Cannot determine path to home directory"))]
    NoWayHome,
}
