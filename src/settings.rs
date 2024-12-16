// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    path::PathBuf,
};
use toml_edit::{
    visit::{visit_inline_table, visit_table_like_kv, Visit},
    Array, InlineTable, Item, Key, Table, Value,
};

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
            ..Default::default()
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

    pub fn to_toml(&self) -> (Key, Item) {
        let mut repo_opts = Table::new();
        let mut bootstrap_opts = Table::new();

        repo_opts.insert("branch", Item::Value(Value::from(&self.branch)));
        repo_opts.insert("remote", Item::Value(Value::from(&self.remote)));

        if let Some(worktree) = &self.worktree {
            repo_opts.insert(
                "worktree",
                Item::Value(Value::from(worktree.to_string_lossy().into_owned())),
            );
        }

        if let Some(bootstrap) = &self.bootstrap {
            bootstrap_opts.insert("clone", Item::Value(Value::from(&bootstrap.clone)));

            if let Some(os) = &bootstrap.os {
                bootstrap_opts.insert("os", Item::Value(Value::from(os.to_string())));
            }

            if let Some(depends) = &bootstrap.depends {
                bootstrap_opts
                    .insert("depends", Item::Value(Value::Array(Array::from_iter(depends))));
            }

            if let Some(ignores) = &bootstrap.ignores {
                bootstrap_opts
                    .insert("ignores", Item::Value(Value::Array(Array::from_iter(ignores))));
            }

            if let Some(users) = &bootstrap.users {
                bootstrap_opts.insert("users", Item::Value(Value::Array(Array::from_iter(users))));
            }

            if let Some(hosts) = &bootstrap.hosts {
                bootstrap_opts.insert("hosts", Item::Value(Value::Array(Array::from_iter(hosts))));
            }

            repo_opts.insert("bootstrap", Item::Table(bootstrap_opts));
        }

        let key = Key::new(&self.name);
        let value = Item::Table(repo_opts);
        (key, value)
    }
}

fn repo_settings_from_toml<'toml>(entry: (&'toml Key, &'toml Item)) -> RepoSettings {
    let (key, value) = entry;
    let mut repo = RepoSettings { name: key.get().into(), ..Default::default() };
    repo.visit_item(value);
    repo
}

impl From<(Key, Item)> for RepoSettings {
    fn from(entry: (Key, Item)) -> Self {
        let (key, value) = entry;
        repo_settings_from_toml((&key, &value))
    }
}

impl<'toml> From<(&'toml Key, &'toml Item)> for RepoSettings {
    fn from(entry: (&'toml Key, &'toml Item)) -> Self {
        repo_settings_from_toml(entry)
    }
}

impl<'toml> Visit<'toml> for RepoSettings {
    fn visit_table_like_kv(&mut self, key: &'toml str, node: &'toml Item) {
        match key {
            "branch" => self.branch = node.as_str().unwrap_or("master").into(),
            "remote" => self.remote = node.as_str().unwrap_or("origin").into(),
            "worktree" => self.worktree = node.as_str().map(|s| s.into()),
            "bootstrap" => {
                let mut bootstrap = BootstrapSettings::default();
                bootstrap.visit_item(node);
                self.bootstrap = Some(bootstrap);
            }
            &_ => visit_table_like_kv(self, key, node),
        };
        visit_table_like_kv(self, key, node);
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
        Self { clone: url.into(), ..Default::default() }
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

impl<'toml> Visit<'toml> for BootstrapSettings {
    fn visit_table_like_kv(&mut self, key: &'toml str, node: &'toml Item) {
        match key {
            "clone" => self.clone = node.as_str().unwrap_or_default().into(),
            "os" => self.os = node.as_str().map(OsKind::from),
            "depends" => {
                self.depends = node.as_array().map(|a| {
                    a.into_iter().map(|s| s.as_str().unwrap_or_default().to_string()).collect()
                })
            }
            "ignores" => {
                self.ignores = node.as_array().map(|a| {
                    a.into_iter().map(|s| s.as_str().unwrap_or_default().to_string()).collect()
                })
            }
            "users" => {
                self.users = node.as_array().map(|a| {
                    a.into_iter().map(|s| s.as_str().unwrap_or_default().to_string()).collect()
                })
            }
            "hosts" => {
                self.hosts = node.as_array().map(|a| {
                    a.into_iter().map(|s| s.as_str().unwrap_or_default().to_string()).collect()
                })
            }
            &_ => visit_table_like_kv(self, key, node),
        };

        visit_table_like_kv(self, key, node);
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

impl From<&str> for OsKind {
    fn from(data: &str) -> Self {
        match data {
            "any" => Self::Any,
            "unix" => Self::Unix,
            "macos" => Self::MacOs,
            "windows" => Self::Windows,
            &_ => Self::Any,
        }
    }
}

impl Display for OsKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            OsKind::Any => write!(f, "any"),
            OsKind::Unix => write!(f, "unix"),
            OsKind::MacOs => write!(f, "macos"),
            OsKind::Windows => write!(f, "windows"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CmdHookSettings {
    pub cmd: String,
    pub hooks: Vec<HookSettings>,
}

impl CmdHookSettings {
    pub fn new(cmd: impl Into<String>) -> Self {
        Self { cmd: cmd.into(), ..Default::default() }
    }

    pub fn add_hook(mut self, hook: HookSettings) -> Self {
        self.hooks.push(hook);
        self
    }

    fn to_toml(&self) -> (Key, Item) {
        let mut tables = Array::new();
        let mut iter = self.hooks.iter().enumerate().peekable();
        while let Some((_, hook)) = iter.next() {
            let mut inline = InlineTable::new();
            let decor = inline.decor_mut();

            decor.set_prefix("\n    ");
            if iter.peek().is_none() {
                decor.set_suffix("\n");
            }

            if let Some(pre) = &hook.pre {
                inline.insert("pre", Value::from(pre));
            }

            if let Some(post) = &hook.post {
                inline.insert("post", Value::from(post));
            }

            if let Some(workdir) = &hook.workdir {
                inline.insert("workdir", Value::from(workdir.to_string_lossy().into_owned()));
            }

            tables.push_formatted(Value::from(inline));
        }

        let key = Key::new(&self.cmd);
        let value = Item::Value(Value::from(tables));
        (key, value)
    }
}

fn cmd_hook_settings_from_toml<'toml>(entry: (&'toml Key, &'toml Item)) -> CmdHookSettings {
    let (key, value) = entry;
    let mut cmd_hook = CmdHookSettings::new(key.get());
    cmd_hook.visit_item(value);
    cmd_hook
}

impl From<(Key, Item)> for CmdHookSettings {
    fn from(entry: (Key, Item)) -> Self {
        let (key, value) = entry;
        cmd_hook_settings_from_toml((&key, &value))
    }
}

impl<'toml> From<(&'toml Key, &'toml Item)> for CmdHookSettings {
    fn from(entry: (&'toml Key, &'toml Item)) -> Self {
        cmd_hook_settings_from_toml(entry)
    }
}

impl<'toml> Visit<'toml> for CmdHookSettings {
    fn visit_inline_table(&mut self, node: &'toml InlineTable) {
        let hook = HookSettings {
            pre: node.get("pre").and_then(|s| s.as_str().map(|s| s.into())),
            post: node.get("post").and_then(|s| s.as_str().map(|s| s.into())),
            workdir: node.get("workdir").and_then(|s| s.as_str().map(|s| s.into())),
        };
        self.hooks.push(hook);

        visit_inline_table(self, node);
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HookSettings {
    pub pre: Option<String>,
    pub post: Option<String>,
    pub workdir: Option<PathBuf>,
}

impl HookSettings {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_pre(mut self, script: impl Into<String>) -> Self {
        self.pre = Some(script.into());
        self
    }

    pub fn with_post(mut self, script: impl Into<String>) -> Self {
        self.post = Some(script.into());
        self
    }

    pub fn with_workdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workdir = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::report;
    use toml_edit::{DocumentMut, TomlError};

    #[fixture]
    fn repo_settings_doc() -> Result<DocumentMut, TomlError> {
        let doc: DocumentMut = indoc! {r#"
            [foo]
            branch = "master"
            remote = "origin"
            worktree = "$HOME"

            [foo.should_ignore_this]
            clone = "https://some/url"
            os = "unix"

            [bar]
            branch = "main"
            remote = "upstream"
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

    #[fixture]
    fn cmd_hook_settings_doc() -> Result<DocumentMut, TomlError> {
        let doc: DocumentMut = indoc! {r#"
            commit = [
                { pre = "hook.sh", post = "hook.sh", workdir = "/some/path" },
                { pre = "hook.sh" },
                { post = "hook.sh" }
            ]
        "#}
        .parse()?;
        Ok(doc)
    }

    #[report]
    #[rstest]
    #[case::no_bootstrap(RepoSettings::new("foo", "master", "origin").with_worktree("$HOME"))]
    #[case::with_bootstrap(
        RepoSettings::new("bar", "main", "upstream")
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

    #[rstest]
    #[case::no_bootstrap(
        RepoSettings::new("foo", "main", "origin").with_worktree("$HOME"),
        indoc! {r#"
            [foo]
            branch = "main"
            remote = "origin"
            worktree = "$HOME"
        "#},
    )]
    #[case::with_bootstrap(
        RepoSettings::new("bar", "main", "upstream")
            .with_worktree("$HOME")
            .with_bootstrap(
                BootstrapSettings::new("https://some/url")
                    .with_os(OsKind::Unix)
                    .with_depends(["baz", "raz"])
                    .with_ignores(["README*", "LICENSE*"])
                    .with_users(["awkless", "lovelace"])
                    .with_hosts(["sedgwick", "dijkstra"])
            ),
        indoc! {r#"
            [bar]
            branch = "main"
            remote = "upstream"
            worktree = "$HOME"

            [bar.bootstrap]
            clone = "https://some/url"
            os = "unix"
            depends = ["baz", "raz"]
            ignores = ["README*", "LICENSE*"]
            users = ["awkless", "lovelace"]
            hosts = ["sedgwick", "dijkstra"]
        "#}
    )]
    fn repo_setings_to_toml_return_key_item(#[case] input: RepoSettings, #[case] expect: &str) {
        let (key, item) = input.to_toml();
        let mut doc = DocumentMut::new();
        let table = doc.as_table_mut();
        table.insert_formatted(&key, item);
        table.set_implicit(true);
        assert_eq!(doc.to_string(), expect);
    }

    #[report]
    #[rstest]
    #[case(
        CmdHookSettings::new("commit")
            .add_hook(
                HookSettings::new()
                    .with_pre("hook.sh")
                    .with_post("hook.sh")
                    .with_workdir("/some/path")
            )
            .add_hook(HookSettings::new().with_pre("hook.sh"))
            .add_hook(HookSettings::new().with_post("hook.sh")),
    )]
    fn cmd_hook_settings_from_key_item_return_self(
        cmd_hook_settings_doc: Result<DocumentMut, TomlError>,
        #[case] expect: CmdHookSettings,
    ) -> Result<(), TomlError> {
        let cmd_hook_settings_doc = cmd_hook_settings_doc?;
        let entry = cmd_hook_settings_doc.as_table().get_key_value(expect.cmd.as_str()).unwrap();
        let result = CmdHookSettings::from(entry);
        assert_eq!(result, expect);

        Ok(())
    }

    #[rstest]
    #[case(
        CmdHookSettings::new("commit")
            .add_hook(
                HookSettings::new()
                    .with_pre("hook.sh")
                    .with_post("hook.sh")
                    .with_workdir("/some/path")
            )
            .add_hook(HookSettings::new().with_pre("hook.sh"))
            .add_hook(HookSettings::new().with_post("hook.sh")),
        indoc! {r#"
            commit = [
                { pre = "hook.sh", post = "hook.sh", workdir = "/some/path" },
                { pre = "hook.sh" },
                { post = "hook.sh" }
            ]
        "#},
    )]
    fn cmd_hook_settings_to_toml_return_key_item(
        #[case] input: CmdHookSettings,
        #[case] expect: &str,
    ) {
        let (key, item) = input.to_toml();
        let mut doc = DocumentMut::new();
        let table = doc.as_table_mut();
        table.insert_formatted(&key, item);
        table.set_implicit(true);
        assert_eq!(doc.to_string(), expect);
    }
}
