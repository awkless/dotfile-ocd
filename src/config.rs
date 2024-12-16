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
pub struct ConfigFile<'cfg, C, L>
where
    C: Config,
    L: Locator,
{
    doc: Toml,
    config: C,
    locator: &'cfg L,
}

impl<'cfg, C, L> ConfigFile<'cfg, C, L>
where
    C: Config,
    L: Locator,
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
    pub fn load(config: C, locator: &'cfg L) -> Result<Self, ConfigError> {
        let path = config.location(locator);
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
        let doc: Toml = buffer.parse().context(TomlSnafu { path: path.to_path_buf() })?;

        Ok(Self { doc, config, locator })
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.config.location(self.locator)
    }
}

impl<'cfg, C, L> Display for ConfigFile<'cfg, C, L>
where
    C: Config,
    L: Locator,
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
    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepoConfig;

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

    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path {
        locator.repos_config()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CmdHookConfig;

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

    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path {
        locator.hooks_config()
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
            .with_file("config.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        # Formatting should remain the same!

                        [repos.vim]
                        branch = "master"
                        remote = "origin"
                        workdir_home = true

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
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    fn config_file_load_parse_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repos_config().return_const(fixture.as_path().into());
        locator.expect_hooks_config().return_const(fixture.as_path().into());

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::hook_cmd_config(CmdHookConfig)]
    fn config_file_load_create_new_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_repos_config().return_const(config_dir.as_path().join("repos.toml"));
        locator.expect_hooks_config().return_const(config_dir.as_path().join("hooks.toml"));

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert!(config.as_path().exists());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    fn config_file_load_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("bad_format.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repos_config().return_const(fixture.as_path().into());
        locator.expect_hooks_config().return_const(fixture.as_path().into());

        let result = ConfigFile::load(config_kind, &locator);
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));

        Ok(())
    }
}
