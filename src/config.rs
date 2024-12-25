// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

mod locate;
mod settings;
mod toml;

#[doc(inline)]
pub use locate::*;
pub use settings::*;
pub use toml::*;

use log::debug;
use mkdirp::mkdirp;
use snafu::prelude::*;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    io::{Error as IoError, Read, Write},
    path::{Path, PathBuf},
    vec::IntoIter as VecIntoIter,
};
use toml_edit::{Item, Key};

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
        let doc = buffer.parse().context(TomlSnafu { path: path.to_path_buf() })?;

        Ok(Self { doc, config, locator })
    }

    /// Save current data to configuration file.
    ///
    /// If path to configuration file does not exist, then it will created at
    /// target location. Otherwise, configuration file will be written to like
    /// normal.
    ///
    /// # Errors
    ///
    /// Will fail if parent directory cannot be created when needed, or
    /// configuration file cannot be opened, or written to for whatever reason.
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

    /// Get configuration setting.
    ///
    /// # Errors
    ///
    /// Will fail if configuration setting does not exist, or target table
    /// setting does not exist or was not defined as a table.
    pub fn get(&self, key: impl AsRef<str>) -> Result<C::Entry, ConfigError> {
        self.config.get(self.locator, &self.doc, key.as_ref())
    }

    /// Add configuration setting.
    ///
    /// If entry already exists, then it will be replaced with the original
    /// entry being returned. Otherwise, the entry will be added in with [`None`]
    /// being returned to indicate that no replacement took place.
    ///
    /// Will create table data if needed in case it does not exist for whatever
    /// reason.
    ///
    /// # Errors
    ///
    /// Will fail if table setting is defined but not defined as a table.
    pub fn add(&mut self, entry: C::Entry) -> Result<Option<C::Entry>, ConfigError> {
        self.config.add(self.locator, &mut self.doc, entry)
    }

    /// Remove configuration setting.
    ///
    /// # Errors
    ///
    /// Will fail if configuration setting does not exist, or target table
    /// setting does not exist or was not defined as a table.
    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<C::Entry, ConfigError> {
        self.config.remove(self.locator, &mut self.doc, key.as_ref())
    }

    /// Return iterator over deserialized settings in configuration file.
    ///
    /// Yields all configuration settings in deserialized form from start to
    /// end.
    pub fn iter(&self) -> ConfigFileIterator<'_, C> {
        let entries = if let Ok(table) = self.doc.get_table(self.config.target_table()) {
            table.iter().map(|(key, value)| (Key::new(key), value.clone())).collect()
        } else {
            Vec::new()
        };

        ConfigFileIterator { config: &self.config, entries: entries.into_iter() }
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.config.location(self.locator)
    }
}

impl<C, L> Display for ConfigFile<'_, C, L>
where
    C: Config,
    L: Locator,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.doc)
    }
}

pub struct ConfigFileIterator<'cfg, C>
where
    C: Config,
{
    config: &'cfg C,
    entries: VecIntoIter<(Key, Item)>,
}

impl<C> Iterator for ConfigFileIterator<'_, C>
where
    C: Config,
{
    type Item = C::Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(|(key, value)| C::Entry::from((key, value)))
    }
}

/// Configuration file startegy.
///
/// Interface to simplify serialization and deserialization of parsed TOML data.
pub trait Config: Debug {
    type Entry: Settings;

    fn get(
        &self,
        locator: &impl Locator,
        doc: &Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError>;

    fn add(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        entry: Self::Entry,
    ) -> Result<Option<Self::Entry>, ConfigError>;

    fn remove(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError>;

    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path;

    fn target_table(&self) -> &str;
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct RepoConfig;

impl Config for RepoConfig {
    type Entry = RepoSettings;

    fn get(
        &self,
        locator: &impl Locator,
        doc: &Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .get(self.target_table(), key)
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?;

        Ok(RepoSettings::from(entry))
    }

    fn add(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        entry: Self::Entry,
    ) -> Result<Option<Self::Entry>, ConfigError> {
        let entry = doc
            .add(self.target_table(), entry.to_toml())
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?
            .map(RepoSettings::from);

        Ok(entry)
    }

    fn remove(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .remove(self.target_table(), key)
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?;

        Ok(RepoSettings::from(entry))
    }

    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path {
        locator.repo_config_file()
    }

    fn target_table(&self) -> &str {
        "repos"
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct CmdHookConfig;

impl Config for CmdHookConfig {
    type Entry = CmdHookSettings;

    fn get(
        &self,
        locator: &impl Locator,
        doc: &Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .get(self.target_table(), key)
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?;

        Ok(CmdHookSettings::from(entry))
    }

    fn add(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        entry: Self::Entry,
    ) -> Result<Option<Self::Entry>, ConfigError> {
        let entry = doc
            .add(self.target_table(), entry.to_toml())
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?
            .map(CmdHookSettings::from);

        Ok(entry)
    }

    fn remove(
        &self,
        locator: &impl Locator,
        doc: &mut Toml,
        key: &str,
    ) -> Result<Self::Entry, ConfigError> {
        let entry = doc
            .remove(self.target_table(), key)
            .context(TomlSnafu { path: self.location(locator).to_path_buf() })?;

        Ok(CmdHookSettings::from(entry))
    }

    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path {
        locator.hook_config_file()
    }

    fn target_table(&self) -> &str {
        "hooks"
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
    use crate::testenv::{FileKind, FixtureHarness};

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::{report, Whatever};

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
                        bare_alias = "$HOME"

                        [hooks]
                        bootstrap = [
                            { pre = "hook.sh", post = "hook.sh", workdir = "/some/dir" },
                            { pre = "hook.sh" }
                        ]
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?
            .with_file("not_table.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        repos = 'not a table'
                        hooks = 'not a table'
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?
            .with_file("bad_format.toml", |fixture| {
                fixture.data("this 'will fail!").kind(FileKind::Normal).write()
            })?;

        Ok(harness)
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_load_parse_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::hook_cmd_config(CmdHookConfig)]
    #[report]
    fn config_file_load_create_new_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(config_dir.as_path().join("repos.toml"));
        locator.expect_hook_config_file().return_const(config_dir.as_path().join("hooks.toml"));

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert!(config.as_path().exists());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_load_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("bad_format.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let result = ConfigFile::load(config_kind, &locator);
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));

        Ok(())
    }

    #[rstest]
    #[case::repo_config(
        RepoConfig,
        RepoSettings::new("dwm", "main", "upstream").with_bare_alias("$HOME")
    )]
    #[case::cmd_hook_config(
        CmdHookConfig,
        CmdHookSettings::new("commit").add_hook(HookSettings::new().with_post("hook.sh")),
    )]
    #[report]
    fn config_file_save_preserves_formatting<E, T>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: T,
        #[case] expect: E,
    ) -> Result<(), Whatever>
    where
        E: Settings,
        T: Config<Entry = E>,
    {
        let mut config_dir = config_dir?;
        let fixture = config_dir.get_mut("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        config.add(expect).with_whatever_context(|_| "Failed to add setting")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        fixture.sync()?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_save_create_new_file(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(config_dir.as_path().join("repos.toml"));
        locator.expect_hook_config_file().return_const(config_dir.as_path().join("hooks.toml"));

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        assert!(config.as_path().exists());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(
        RepoConfig,
        "vim",
        RepoSettings::new("vim", "master", "origin").with_bare_alias("$HOME")
    )]
    #[case::repo_config(
        CmdHookConfig,
        "bootstrap",
        CmdHookSettings::new("bootstrap")
            .add_hook(
                HookSettings::new()
                    .with_pre("hook.sh")
                    .with_post("hook.sh")
                    .with_workdir("/some/dir")
            )
            .add_hook(HookSettings::new().with_pre("hook.sh")),
    )]
    #[report]
    fn config_file_get_return_setting<E, T>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: T,
        #[case] key: &str,
        #[case] expect: E,
    ) -> Result<(), Whatever>
    where
        E: Settings,
        T: Config<Entry = E>,
    {
        let config_dir = config_dir?;
        let fixture = config_dir.get("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.get(key).with_whatever_context(|_| "Failed to get setting")?;
        assert_eq!(result, expect);

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_get_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.get("non-existent");
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));

        Ok(())
    }

    #[rstest]
    #[case::repo_config(
        RepoConfig,
        RepoSettings::new("dwm", "main", "upstream").with_bare_alias("$HOME")
    )]
    #[case::cmd_hook_config(
        CmdHookConfig,
        CmdHookSettings::new("commit").add_hook(HookSettings::new().with_post("hook.sh")),
    )]
    #[report]
    fn config_file_new_return_none<E, T>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: T,
        #[case] entry: E,
    ) -> Result<(), Whatever>
    where
        E: Settings,
        T: Config<Entry = E>,
    {
        let mut config_dir = config_dir?;
        let fixture = config_dir.get_mut("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.add(entry).with_whatever_context(|_| "Failed to add setting")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        fixture.sync()?;
        assert_eq!(result, None);
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(
        RepoConfig,
        RepoSettings::new("vim", "main", "upstream").with_bare_alias("$HOME"),
        Some(RepoSettings::new("vim", "master", "origin").with_bare_alias("$HOME")),
    )]
    #[case::cmd_hook_config(
        CmdHookConfig,
        CmdHookSettings::new("bootstrap")
            .add_hook(HookSettings::new().with_pre("new_hook.sh").with_post("new_hook.sh"))
            .add_hook(HookSettings::new().with_pre("new_hook.sh").with_workdir("/new/dir")),
        Some(CmdHookSettings::new("bootstrap")
            .add_hook(
                HookSettings::new()
                    .with_pre("hook.sh")
                    .with_post("hook.sh")
                    .with_workdir("/some/dir")
            )
            .add_hook(HookSettings::new().with_pre("hook.sh"))),
    )]
    #[report]
    fn config_file_new_return_some<E, T>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: T,
        #[case] entry: E,
        #[case] expect: Option<E>,
    ) -> Result<(), Whatever>
    where
        E: Settings,
        T: Config<Entry = E>,
    {
        let mut config_dir = config_dir?;
        let fixture = config_dir.get_mut("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.add(entry).with_whatever_context(|_| "Failed to add setting")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        fixture.sync()?;
        assert_eq!(result, expect);
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_add_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let mut config_dir = config_dir?;
        let fixture = config_dir.get_mut("not_table.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.add(Default::default());
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));
        Ok(())
    }

    #[rstest]
    #[case::repo_config(
        RepoConfig,
        "vim",
        RepoSettings::new("vim", "master", "origin").with_bare_alias("$HOME"),
    )]
    #[case::cmd_hook_config(
        CmdHookConfig,
        "bootstrap",
        CmdHookSettings::new("bootstrap")
            .add_hook(
                HookSettings::new()
                    .with_pre("hook.sh")
                    .with_post("hook.sh")
                    .with_workdir("/some/dir")
            )
            .add_hook(HookSettings::new().with_pre("hook.sh")),
    )]
    #[report]
    fn config_file_remove_return_deleted_setting<E, T>(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: T,
        #[case] key: &str,
        #[case] expect: E,
    ) -> Result<(), Whatever>
    where
        E: Settings,
        T: Config<Entry = E>,
    {
        let mut config_dir = config_dir?;
        let fixture = config_dir.get_mut("config.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.remove(key).with_whatever_context(|_| "Failed to remove setting")?;
        config.save().with_whatever_context(|_| "Failed to save configuration file")?;
        fixture.sync()?;
        assert_eq!(result, expect);
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }

    #[rstest]
    #[case::repo_config(RepoConfig)]
    #[case::cmd_hook_config(CmdHookConfig)]
    #[report]
    fn config_file_remove_return_err_toml(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] config_kind: impl Config,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("not_table.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_hook_config_file().return_const(fixture.as_path().into());

        let mut config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        let result = config.remove("fail");
        assert!(matches!(result.unwrap_err().0, InnerConfigError::Toml { .. }));

        Ok(())
    }
}
