// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::{
    locate::Locator,
    toml::{Toml, TomlError},
    settings::{RepoSettings, CmdHookSettings},
};

use snafu::prelude::*;
use std::{
    path::{PathBuf, Path},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    io::{Read, Write, Error as IoError},
};
use log::debug;
use mkdirp::mkdirp;

/// Format preserving configuration file handler.
#[derive(Clone, Debug)]
pub struct ConfigFile<C>
where
    C: Config,
{
    doc: Toml,
    config: C,
}

impl<C> ConfigFile<C>
where
    C: Config,
{
    /// Load new configuration file.
    ///
    /// If path to configuration file does not exist, then it will created at
    /// target location. Otherwise, configuration file will be read and parsed
    /// like normal.
    ///
    /// # Errors
    ///
    /// Will fail if parent directory cannot be created when needed, or
    /// configuration file cannot be opened, read, and/or parsed at all.
    pub fn load(config: C) -> Result<Self, ConfigError> {
        let path = config.as_path();
        debug!("Load new configuration file from '{}'", path.display());
        let root = path.parent().unwrap();
        mkdirp(root).context(MakeDirPSnafu { path: root.to_path_buf() })?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(false)
            .read(true)
            .create(true)
            .open(path)
            .context(FileOpenSnafu { path: path.to_path_buf() })?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).context(FileReadSnafu { path: path.to_path_buf() })?;
        let doc = buffer.parse().context(TomlSnafu { path: path.to_path_buf() })?;

        Ok(Self { doc, config })
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.config.as_path()
    }
}

impl<C> Display for ConfigFile<C>
where
    C: Config,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.doc)
    }
}

/// Configuration file startegy.
///
/// Interface to simplify serialization and deserialization of parsed TOML data.
pub trait Config: Debug {
    type Entry;

    fn get(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn add(&self, doc: &Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError>;
    fn remove(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn with_locator<'cfg>(&mut self, locator: &'cfg impl Locator);
    fn as_path(&self) -> &Path;
}

#[derive(Clone, Default, Debug)]
pub struct RepoConfig {
    path: PathBuf,
}

impl RepoConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Config for RepoConfig {
    type Entry = RepoSettings;

    fn get(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        todo!();
    }

    fn add(&self, doc: &Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError> {
        todo!();
    }

    fn remove(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        todo!();
    }

    fn as_path(&self) -> &Path {
        &self.path
    }

    fn with_locator<'cfg>(&mut self, locator: &'cfg impl Locator) {
        self.path = locator.config_dir().join(&self.path);
    }
}

#[derive(Clone, Default, Debug)]
pub struct CmdHookConfig {
    path: PathBuf,
}

impl CmdHookConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Config for CmdHookConfig {
    type Entry = CmdHookSettings;

    fn get(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        todo!();
    }

    fn add(&self, doc: &Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError> {
        todo!();
    }

    fn remove(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        todo!();
    }

    fn as_path(&self) -> &Path {
        &self.path
    }

    fn with_locator<'cfg>(&mut self, locator: &'cfg impl Locator) {
        self.path = locator.config_dir().join(&self.path);
    }
}

/// Configuration error type for public API.
#[derive(Debug, Snafu)]
pub struct ConfigError(InnerConfigError);

/// Alias to allow one-off functions with different error type.
pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerConfigError {
    #[snafu(display("Failed to create parent path to '{}'", path.display()))]
    MakeDirP { path: PathBuf, source: IoError },

    #[snafu(display("Failed to open '{}'", path.display()))]
    FileOpen { path: PathBuf, source: IoError },

    #[snafu(display("Failed to read '{}'", path.display()))]
    FileRead { path: PathBuf, source: IoError },

    #[snafu(display("Failed to parse '{}'", path.display()))]
    Toml { path: PathBuf, source: TomlError },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        testenv::{FixtureHarness, FileKind},
        locate::MockLocator,
    };

    use pretty_assertions::assert_eq;
    use snafu::{report, Whatever};
    use rstest::{fixture, rstest};
    use indoc::indoc;

    #[fixture]
    fn config_dir() -> Result<FixtureHarness, Whatever> {
        let harness = FixtureHarness::open()?
            .with_file("repos.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        # Formatting should remain the same!
                        [repos.vim]
                        branch = "master"
                        remote = "origin"
                        workdir_home = true
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?
            .with_file("hooks.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        # Formatting should remain the same!
                        [hooks]
                        commit = [
                            { pre = "hook.sh", post = "hook.sh", workdir = "/some/path" },
                            { pre = "hook.sh" },
                            { post = "hook.sh" }
                        ]
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?
            .with_file("bad_format.toml", |fixture| {
                fixture.data("this 'will fail!").kind(FileKind::Normal).write()
            })?;
        Ok(harness)
    }

    #[report]
    #[rstest]
    #[case::repo_config(RepoConfig::new("repos.toml"), "repos.toml")]
    #[case::cmd_hook_config(CmdHookConfig::new("hooks.toml"), "hooks.toml")]
    fn config_file_load_parse_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] mut config_strat: impl Config,
        #[case] fixture_name: &str,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get(fixture_name)?;
        let mut locator = MockLocator::new();
        locator.expect_config_dir().return_const(config_dir.as_path().into());
        config_strat.with_locator(&locator);

        let config = ConfigFile::load(config_strat)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::repo_config(RepoConfig::new("repos.toml"))]
    #[case::hook_cmd_config(CmdHookConfig::new("hooks.toml"))]
    fn config_file_load_create_new_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] mut config_strat: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_config_dir().return_const(config_dir.as_path().into());
        config_strat.with_locator(&locator);

        let config = ConfigFile::load(config_strat)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert!(config.as_path().exists());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::repo_config(RepoConfig::new("bad_format.toml"))]
    #[case::cmd_hook_config(CmdHookConfig::new("bad_format.toml"))]
    fn config_file_load_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] mut config_strat: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_config_dir().return_const(config_dir.as_path().into());
        config_strat.with_locator(&locator);

        let result = ConfigFile::load(config_strat);
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));

        Ok(())
    }
}
