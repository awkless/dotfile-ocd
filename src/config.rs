// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::{
    locate::Locator,
    toml::Toml,
    settings::{RepoSettings, CmdHookSettings},
};

use snafu::prelude::*;
use std::{
    path::Path,
    fmt::{Display, Formatter, Result as FmtResult},

};
use log::debug;
use mkdirp::mkdirp;

pub trait Config {
    type Entry;

    fn get(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn add(&self, doc: &Toml, entry: Self::Entry) -> Result<Option<Self::Entry>, ConfigError>;
    fn remove(&self, doc: &Toml, key: &str) -> Result<Self::Entry, ConfigError>;
    fn location<'cfg>(&self, locator: &'cfg impl Locator) -> &'cfg Path;
}

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
    pub fn load(config: C, locator: &'cfg L) -> Result<Self, ConfigError> {
        todo!();
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
        todo!();
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
        todo!();
    }
}

#[derive(Debug, Snafu, PartialEq, Eq)]
pub struct ConfigError(InnerConfigError);

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, Snafu, PartialEq, Eq)]
enum InnerConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        testenv::{FixtureHarness, FileKind},
        locate::MockLocator,
    };

    use snafu::{report, Whatever};
    use rstest::{fixture, rstest};
    use indoc::indoc;

    #[fixture]
    fn config_dir() -> Result<FixtureHarness, Whatever> {
        let harness = FixtureHarness::open()?
            .with_file("config.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        [repos.foo]
                        branch = "master"
                        remote = "origin"
                        worktree = "$HOME"

                        [repos.bar]
                        branch = "main"
                        remote = "upstream"
                        worktree = "$HOME"

                        [repos.bar.bootstrap]
                        clone = "https://some/url"
                        os = "unix"
                        depends = ["foo", "baz"]
                        ignores = ["LICENSE*", "README*"]
                        users = ["awkless", "sedgwick"]
                        hosts = ["lovelace", "turing"]

                        [hooks]
                        commit = [
                            { pre = "hook.sh", post = "hook.sh", workdir = "/some/path" },
                            { pre = "hook.sh" },
                            { post = "hook.sh" }
                        ]
                    "#})
                    .kind(FileKind::Normal)
                    .write()
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
        locator.expect_config_dir().return_const(config_dir.as_path().into());

        let config = ConfigFile::load(config_kind, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;
        assert_eq!(config.to_string(), fixture.as_str());

        Ok(())
    }
}
