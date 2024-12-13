// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use toml_edit::{Key, Item};

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct RepoSettings {
    pub name: String,
    pub branch: String,
    pub remote: String,
    pub worktree: Option<PathBuf>,
    pub bootstrap: Option<BootstrapSettings>,
}

impl RepoSettings {
    pub fn new(
        name: impl Into<String>,
        branch: impl Into<String>,
        remote: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            branch: branch.into(),
            remote: remote.into(),
            worktree: Default::default(),
            bootstrap: Default::default(),
        }
    }

    pub fn with_worktree(mut self, worktree: impl Into<PathBuf>) -> Self {
        self.worktree = Some(worktree.into());
        self
    }

    pub fn with_bootstrap(mut self, bootstrap: BootstrapSettings) -> Self {
        self.bootstrap = Some(bootstrap);
        self
    }
}

impl<'toml> From<(&'toml Key, &'toml Item)> for RepoSettings {
    fn from(entry: (&'toml Key, &'toml Item)) -> Self {
        todo!();
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct BootstrapSettings {
    pub clone: String,
    pub os: Option<OsKind>,
    pub depends: Option<Vec<String>>,
    pub ignores: Option<Vec<String>>,
    pub users: Option<Vec<String>>,
    pub hosts: Option<Vec<String>>,
}

impl BootstrapSettings {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            clone: url.into(),
            os: Default::default(),
            depends: Default::default(),
            ignores: Default::default(),
            users: Default::default(),
            hosts: Default::default(),
        }
    }

    pub fn with_os(mut self, kind: OsKind) -> Self {
        self.os = Some(kind);
        self
    }

    pub fn with_depends(mut self, depends: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut vec = Vec::new();
        vec.extend(depends.into_iter().map(Into::into));
        self.depends = Some(vec);
        self
    }

    pub fn with_ignores(mut self, ignores: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut vec = Vec::new();
        vec.extend(ignores.into_iter().map(Into::into));
        self.ignores = Some(vec);
        self
    }

    pub fn with_users(mut self, users: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut vec = Vec::new();
        vec.extend(users.into_iter().map(Into::into));
        self.users = Some(vec);
        self
    }

    pub fn with_hosts(mut self, hosts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut vec = Vec::new();
        vec.extend(hosts.into_iter().map(Into::into));
        self.hosts = Some(vec);
        self
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub enum OsKind {
    #[default]
    Any,

    Unix,

    MacOs,

    Windows,
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::report;
    use toml_edit::{DocumentMut, TomlError};
    use indoc::indoc;

    #[fixture]
    fn repo_settings_doc() -> Result<DocumentMut, TomlError> {
        let doc: DocumentMut = indoc! {r#"
            [foo]
            branch = "master"
            remote = "origin"
            worktree = "$HOME"

            [bar]
            branch = "main"
            remote = "origin"
            worktree = "$HOME"

            [bar.bootstrap]
            clone = "https://some/url"
            os = "unix"
            depends = ["foo", "baz"]
            ignores = ["LICENSE*", "README*"]
            users = ["awkless", "sedgwick"]
            hosts = ["lovelace", "turing"]
        "#}
        .parse()?;
        Ok(doc)
    }

    #[report]
    #[rstest]
    #[case::no_bootstrap(
        RepoSettings::new("foo", "master", "origin")
            .with_worktree("$HOME")
    )]
    #[case::with_bootstrap(
        RepoSettings::new("bar", "main", "origin")
            .with_worktree("$HOME")
            .with_bootstrap(
                BootstrapSettings::new("https://some/url")
                    .with_os(OsKind::Unix)
                    .with_depends(["foo", "baz"])
                    .with_ignores(["LICENSE*", "README*"])
                    .with_users(["awkless", "sedgwick"])
                    .with_hosts(["lovelace", "turing"])
            )
    )]
    fn repo_settings_from_key_item_return_self(
        repo_settings_doc: Result<DocumentMut, TomlError>,
        #[case] expect: RepoSettings,
    ) -> Result<(), TomlError> {
        let repo_settings_doc = repo_settings_doc?;
        let entry = repo_settings_doc.as_table().get_key_value(expect.name.as_str()).unwrap();
        let result = RepoSettings::from(entry);
        assert_eq!(result, expect);

        Ok(())
    }
}
