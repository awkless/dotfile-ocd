// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::{
    locate::Locator,
    settings::{CmdHookSettings, RepoSettings, Settings},
    toml::{Toml, TomlError},
};

use log::debug;
use mkdirp::mkdirp;
use snafu::prelude::*;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    io::{Error as IoError, Read, Write},
    path::{Path, PathBuf},
};

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

    pub fn save(&mut self) -> Result<(), ConfigError> {
        let path = self.as_path();
        debug!("Save configuration manager data to '{}'", self.as_path().display());
        let root = path.parent().unwrap();
        mkdirp(root).context(MakeDirPSnafu { path: root.to_path_buf() })?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.as_path())
            .context(FileOpenSnafu { path: path.to_path_buf() })?;
        let buffer = self.doc.to_string();
        file.write_all(buffer.as_bytes()).context(FileWriteSnafu { path: path.to_path_buf() })?;

        Ok(())
    }

    pub fn get(&self, key: impl AsRef<str>) -> Result<C::Entry, ConfigError> {
        self.config.get(&self.doc, key.as_ref())
    }

    pub fn add(&mut self, entry: C::Entry) -> Result<Option<C::Entry>, ConfigError> {
        self.config.add(&mut self.doc, entry)
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<C::Entry, ConfigError> {
        self.config.remove(&mut self.doc, key.as_ref())
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
    type Entry: Settings;

    fn get(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn add(&self, doc: &mut Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError>;
    fn remove(&self, doc: &mut Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn as_path(&self) -> &Path;
    fn with_locator<'cfg>(&mut self, locator: &'cfg impl Locator);
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
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
        let entry = doc
            .get("repos", key)
            .context(TomlSnafu { path: self.path.to_path_buf() })?;

        Ok(RepoSettings::from(entry))
    }

    fn add(&self, doc: &mut Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError> {
        let entry = doc
            .add("repos", entry.to_toml())
            .context(TomlSnafu { path: self.path.to_path_buf() })?
            .map(RepoSettings::from);
        Ok(entry)
    }

    fn remove(&self, doc: &mut Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .remove("repos", key)
            .context(TomlSnafu { path: self.path.to_path_buf() })?;

        Ok(RepoSettings::from(entry))
    }

    fn as_path(&self) -> &Path {
        &self.path
    }

    fn with_locator<'cfg>(&mut self, locator: &'cfg impl Locator) {
        self.path = locator.config_dir().join(&self.path);
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
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
        let entry = doc
            .get("hooks", key)
            .context(TomlSnafu { path: self.path.to_path_buf() })?;

        Ok(CmdHookSettings::from(entry))
    }

    fn add(&self, doc: &mut Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError> {
        let entry = doc
            .add("hooks", entry.to_toml())
            .context(TomlSnafu { path: self.path.to_path_buf() })?
            .map(CmdHookSettings::from);

        Ok(entry)
    }

    fn remove(&self, doc: &mut Toml, key: &str) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .remove("hooks", key)
            .context(TomlSnafu { path: self.path.to_path_buf() })?;

        Ok(CmdHookSettings::from(entry))
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

    #[snafu(display("Failed to write '{}'", path.display()))]
    FileWrite { path: PathBuf, source: IoError },

    #[snafu(display("Failed to parse '{}'", path.display()))]
    Toml { path: PathBuf, source: TomlError },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        locate::MockLocator,
        testenv::{FileKind, FixtureHarness},
    };

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::{report, Whatever};

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

    #[report]
    #[rstest]
    #[case::repo_config(
        RepoConfig::new("repos.toml"),
        "repos.toml",
        RepoSettings::new("dwm", "main", "upstream").with_worktree("$HOME"),
    )]
    fn config_file_save_preserves_formatting<E: Settings, T: Config<Entry = E>>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] mut config_strat: T,
        #[case] fixture_name: &str,
        #[case] setting: E,
    ) -> Result<(), Whatever> {
        let mut config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_config_dir().return_const(config_dir.as_path().to_path_buf());
        config_strat.with_locator(&locator);

        let mut config = ConfigFile::load(config_strat)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        config.add(setting).with_whatever_context(|_| "Failed to add setting")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        config_dir.sync_tracked()?;
        let fixture = config_dir.get(fixture_name)?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }
}
