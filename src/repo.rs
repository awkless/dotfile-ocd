// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::{
    config::{ConfigFile, RepoConfig, Locator},
    vcs::Git,
};

/// Manage repository collection.
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
    pub fn new(config: ConfigFile<'repo, RepoConfig, L>, locator: &'repo L) -> Self {
        Self { git: Git::new(), config, locator }
    }
}
