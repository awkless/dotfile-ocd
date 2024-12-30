// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

mod deps;
mod vcs;

#[doc(inline)]
pub use deps::*;
pub use vcs::*;

use crate::config::{ConfigError, ConfigFile, Locator, RepoConfig, RepoSettings};

use snafu::prelude::*;
use std::{collections::HashSet, path::PathBuf};

/// Manage repository collection.
#[derive(Debug)]
pub struct RepoManager<'repo, L>
where
    L: Locator,
{
    git: Git,
    config: ConfigFile<'repo, RepoConfig, L>,
    locator: &'repo L,
    deps: Dependencies,
}

impl<'repo, L> RepoManager<'repo, L>
where
    L: Locator,
{
    /// Construct new repository manager.
    ///
    /// # Errors
    ///
    /// Will fail if duplicate entries are found in array values of bootstrap
    /// configuration field, or a circular dependency is found.
    pub fn manage(
        config: ConfigFile<'repo, RepoConfig, L>,
        locator: &'repo L,
    ) -> Result<Self, RepoManagerError> {
        duplicate_settings_check(&config)?;

        let mut deps = Dependencies::new();
        deps.with_config_file(&config);
        deps.acyclic_check().context(DependencySnafu)?;

        Ok(Self { git: Git::new(), config, locator, deps })
    }

    /// Initialize new repository.
    ///
    /// Initialize repository through Git, and add an entry for it in special
    /// configuration file.
    ///
    /// # Errors
    ///
    /// Will fail if new repository cannot be initialized, or added into
    /// configuration file.
    pub fn init(
        &mut self,
        name: String,
        branch: Option<String>,
        bare_alias: Option<PathBuf>,
    ) -> Result<(), RepoManagerError> {
        let branch = if let Some(branch) = branch { branch.to_string() } else { "master".into() };
        self.git.with_arg("init");

        let mut repo = RepoSettings::new(&name, &branch, "origin");
        if let Some(alias) = bare_alias {
            repo = repo.with_bare_alias(alias.to_path_buf());
            self.git.with_arg("--bare");
        }

        let dir_path = self.locator.repos_dir().join(&name).to_string_lossy().into_owned();
        self.git.with_args(["--initial-branc", &branch, &dir_path]);
        self.git.run().context(GitSnafu)?;
        self.config.add(repo).context(ConfigFileSnafu)?;
        self.config.save().context(ConfigFileSnafu)?;

        Ok(())
    }
}

fn duplicate_settings_check(
    config: &ConfigFile<'_, RepoConfig, impl Locator>,
) -> Result<(), InnerRepoManagerError> {
    let repos: Vec<RepoSettings> = config.iter().collect();
    for repo in repos {
        if let Some(bootstrap) = repo.bootstrap {
            find_duplicates(&bootstrap.depends, &format!("{}.bootstrap.depends", repo.name))?;
            find_duplicates(&bootstrap.ignores, &format!("{}.bootstrap.ignores", repo.name))?;
            find_duplicates(&bootstrap.users, &format!("{}.bootstrap.users", repo.name))?;
            find_duplicates(&bootstrap.hosts, &format!("{}.bootstrap.hosts", repo.name))?;
        }
    }

    Ok(())
}

fn find_duplicates(
    entries: &Option<Vec<String>>,
    setting_name: &str,
) -> Result<(), InnerRepoManagerError> {
    if let Some(entries) = entries {
        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();

        for entry in entries {
            if !seen.insert(entry) {
                duplicates.push(entry.clone());
            }
        }

        if !duplicates.is_empty() {
            return Err(InnerRepoManagerError::DuplicateSettingValues {
                setting: setting_name.to_string(),
                duplicates,
            });
        }
    }

    Ok(())
}

#[derive(Debug, Snafu)]
pub struct RepoManagerError(InnerRepoManagerError);

pub type Result<T, E = RepoManagerError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerRepoManagerError {
    #[snafu(display("Configuration file management failure"))]
    ConfigFile { source: ConfigError },

    #[snafu(display("Dependency management failure"))]
    Dependency { source: DependencyError },

    #[snafu(display("Git system call failure"))]
    Git { source: GitError },

    #[snafu(display("Repository setting '{setting}' contains duplicate entries: '{:?}'"))]
    DuplicateSettingValues { setting: String, duplicates: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        config::MockLocator,
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
    fn repo_manager_manage_duplicate_settings_check_return_ok(
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
    fn repo_manager_manage_duplicate_settings_check_return_err(
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

    #[report]
    #[rstest]
    #[case::use_defaults("foo".to_string(), None, None)]
    #[case::set_branch("bar".to_string(), Some("main".to_string()), None)]
    #[case::set_bare_alias("baz".to_string(), Some("patch".to_string()), Some("/some/path".into()))]
    fn repo_manager_init_add_repo_to_config_and_repo_dir(
        config_dir: Result<FixtureHarness, Whatever>,
        #[case] repo_name: String,
        #[case] branch: Option<String>,
        #[case] bare_alias: Option<PathBuf>,
    ) -> Result<(), Whatever> {
        let mut config_dir = config_dir?;
        let repos_dir = config_dir.as_path().join("repos");
        let fixture = config_dir.get_mut("repos.toml")?;
        let mut locator = MockLocator::new();
        locator.expect_repo_config_file().return_const(fixture.as_path().into());
        locator.expect_repos_dir().return_const(repos_dir.clone());
        let config = ConfigFile::load(RepoConfig, &locator)
            .with_whatever_context(|_| "Failed to load configuration file")?;

        let mut repo_mgr = RepoManager::manage(config, &locator)
            .with_whatever_context(|_| "Failed to construct repository manager")?;
        repo_mgr
            .init(repo_name.clone(), branch, bare_alias)
            .with_whatever_context(|_| "Failed to initialize new repository")?;
        fixture.sync()?;
        assert!(repos_dir.join(repo_name).exists());
        assert_eq!(repo_mgr.config.to_string(), fixture.as_str());

        Ok(())
    }
}
