// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::{
    config::{ConfigError, ConfigFile, Locator, RepoConfig, RepoSettings},
    vcs::Git,
};

use snafu::prelude::*;
use std::collections::HashMap;

/// Manage repository collection.
#[derive(Debug)]
pub struct RepoManager<'repo, L>
where
    L: Locator,
{
    git: Git,
    config: ConfigFile<'repo, RepoConfig, L>,
    locator: &'repo L,
}

impl<'repo, L> RepoManager<'repo, L>
where
    L: Locator,
{
    /// Construct new repository manager.
    pub fn manage(
        config: ConfigFile<'repo, RepoConfig, L>,
        locator: &'repo L,
    ) -> Result<Self, RepoManagerError> {
        let repo_mgr = Self { git: Git::new(), config, locator };
        repo_mgr.check_duplicate_setting_array_values()?;

        Ok(repo_mgr)
    }

    fn check_duplicate_setting_array_values(&self) -> Result<(), InnerRepoManagerError> {
        let repos: Vec<RepoSettings> = self.config.iter().collect();
        for repo in repos {
            if let Some(bootstrap) = repo.bootstrap {
                let result = find_duplicates(&bootstrap.depends.unwrap_or_default());
                if !result.is_empty() {
                    return Err(InnerRepoManagerError::DuplicateSettingValues {
                        setting: format!("{}.bootstrap.depends", repo.name),
                        duplicates: result,
                    });
                }

                let result = find_duplicates(&bootstrap.ignores.unwrap_or_default());
                if !result.is_empty() {
                    return Err(InnerRepoManagerError::DuplicateSettingValues {
                        setting: format!("{}.bootstrap.ignores", repo.name),
                        duplicates: result,
                    });
                }

                let result = find_duplicates(&bootstrap.users.unwrap_or_default());
                if !result.is_empty() {
                    return Err(InnerRepoManagerError::DuplicateSettingValues {
                        setting: format!("{}.bootstrap.users", repo.name),
                        duplicates: result,
                    });
                }

                let result = find_duplicates(&bootstrap.hosts.unwrap_or_default());
                if !result.is_empty() {
                    return Err(InnerRepoManagerError::DuplicateSettingValues {
                        setting: format!("{}.bootstrap.hosts", repo.name),
                        duplicates: result,
                    });
                }
            }
        }

        Ok(())
    }
}

fn find_duplicates(entries: &Vec<String>) -> Vec<String> {
    let mut values: HashMap<String, usize> = HashMap::new();
    let mut duplicates = Vec::new();
    for entry in entries {
        match values.get_key_value(entry) {
            Some((key, value)) => {
                if *value >= 1 {
                    duplicates.push(entry.clone());
                } else {
                    values.insert(key.clone(), value + 1);
                }
            }
            None => _ = values.insert(entry.clone(), 1),
        };
    }

    duplicates
}

#[derive(Debug, Snafu)]
pub struct RepoManagerError(InnerRepoManagerError);

pub type Result<T, E = RepoManagerError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerRepoManagerError {
    ConfigFile {
        source: ConfigError,
    },

    #[snafu(display("Repository setting '{setting}' contains duplicate entries: '{:}'"))]
    DuplicateSettingValues {
        setting: String,
        duplicates: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        config::MockLocator,
        testenv::{FileKind, FixtureHarness},
    };

    use indoc::indoc;
    use rstest::{fixture, rstest};
    use snafu::{report, Whatever};

    #[fixture]
    fn config_dir() -> Result<FixtureHarness, Whatever> {
        let harness = FixtureHarness::open()?
            .with_file("repos.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        [repos.vim]
                        branch = "master"
                        remote = "origin"
                        bare_alias = "$HOME"

                        [repos.vim.bootstrap]
                        clone = "https://some/url"
                        os = "unix"
                        depends = ["foo", "baz"]
                        ignores = ["LICENSE*", "README*"]
                        users = ["awkless", "lovelace"]
                        hosts = ["lovelace", "turing"]
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?
            .with_file("duplicates.toml", |fixture| {
                fixture
                    .data(indoc! {r#"
                        [repos.vim]
                        branch = "master"
                        remote = "origin"
                        bare_alias = "$HOME"

                        [repos.vim.bootstrap]
                        clone = "https://some/url"
                        os = "unix"
                        depends = ["foo", "baz"]
                        ignores = ["LICENSE*", "README*"]
                        users = ["awkless", "awkless"]
                        hosts = ["lovelace", "turing"]
                    "#})
                    .kind(FileKind::Normal)
                    .write()
            })?;

        Ok(harness)
    }

    #[report]
    #[rstest]
    fn repo_manager_manage_check_duplicate_settings_return_self(
        config_dir: Result<FixtureHarness, Whatever>,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("repos.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        let config = ConfigFile::load(RepoConfig, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;

        let result = RepoManager::manage(config, &locator);
        assert!(result.is_ok());

        Ok(())
    }

    #[report]
    #[rstest]
    fn repo_manager_manage_check_duplicate_settings_return_err(
        config_dir: Result<FixtureHarness, Whatever>,
    ) -> Result<(), Whatever> {
        let config_dir = config_dir?;
        let fixture = config_dir.get("duplicates.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        let config = ConfigFile::load(RepoConfig, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;

        let result = RepoManager::manage(config, &locator);
        assert!(matches!(
            result.unwrap_err().0,
            InnerRepoManagerError::DuplicateSettingValues { .. }
        ));

        Ok(())
    }
}
