// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::cli::*;

use clap::ValueEnum;
use std::path::PathBuf;


#[derive(Debug, Eq, PartialEq)]
pub enum Ctx {
    Init(InitCtx),
    Clone(CloneCtx),
    Remove(RemoveCtx),
    Deploy(DeployCtx),
    Undeploy(UndeployCtx),
    List(ListCtx),
    Status(StatusCtx),
    Pull(PullCtx),
    Push(PushCtx),
    Commit(CommitCtx),
    Git(GitCtx),
}

impl From<Cli> for Ctx {
    fn from(opts: Cli) -> Self {
        match opts.cmd_set {
            CommandSet::Init(_) => Self::Init(InitCtx::from(opts)),
            CommandSet::Clone(_) => Self::Clone(CloneCtx::from(opts)),
            CommandSet::Remove(_) => Self::Remove(RemoveCtx::from(opts)),
            CommandSet::Deploy(_) => Self::Deploy(DeployCtx::from(opts)),
            CommandSet::Undeploy(_) => Self::Undeploy(UndeployCtx::from(opts)),
            CommandSet::List(_) => Self::List(ListCtx::from(opts)),
            CommandSet::Status(_) => Self::Status(StatusCtx::from(opts)),
            CommandSet::Pull(_) => Self::Pull(PullCtx::from(opts)),
            CommandSet::Push(_) => Self::Push(PushCtx::from(opts)),
            CommandSet::Commit(_) => Self::Commit(CommitCtx::from(opts)),
            CommandSet::Git(_) => Self::Git(GitCtx::from(opts)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct InitCtx {
    pub name: String,
    pub bare_alias: Option<PathBuf>,
    pub branch: Option<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for InitCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Init(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'init'"),
        };

        Self {
            name: cmd_set.name,
            bare_alias: cmd_set.bare_alias,
            branch: cmd_set.branch,
            shared: shared_opts.into(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CloneCtx {
    pub remote: String,
    pub repo: Option<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for CloneCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Clone(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'clone'"),
        };

        Self { remote: cmd_set.remote, repo: cmd_set.repo, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RemoveCtx {
    pub remote: String,
    pub shared: SharedCtx,
}

impl From<Cli> for RemoveCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Remove(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'remove'"),
        };

        Self { remote: cmd_set.remote, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeployCtx {
    pub repos: Vec<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for DeployCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Deploy(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'deploy'"),
        };

        Self { repos: cmd_set.repos, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct UndeployCtx {
    pub repos: Vec<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for UndeployCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Undeploy(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'undeploy'"),
        };

        Self { repos: cmd_set.repos, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ListCtx {
    pub show: ListAction,
    pub shared: SharedCtx,
}

impl From<Cli> for ListCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::List(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'list'"),
        };

        Self { show: cmd_set.show, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatusCtx {
    pub terse: bool,
    pub shared: SharedCtx,
}

impl From<Cli> for StatusCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Status(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'status'"),
        };

        Self { terse: cmd_set.terse, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PullCtx {
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for PullCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Pull(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'pull'"),
        };

        Self { remote: cmd_set.remote, branch: cmd_set.branch, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PushCtx {
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for PushCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Push(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'push'"),
        };

        Self { remote: cmd_set.remote, branch: cmd_set.branch, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommitCtx {
    pub fixup: Option<FixupAction>,
    pub message: Option<String>,
    pub shared: SharedCtx,
}

impl From<Cli> for CommitCtx {
    fn from(opts: Cli) -> Self {
        let Cli { shared_opts, cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Commit(opts) => opts,
            _ => unreachable!("This should not happen. The command is not 'commit'"),
        };

        Self { fixup: cmd_set.fixup, message: cmd_set.message, shared: shared_opts.into() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GitCtx {
    pub repo: OsString,
    pub git_args: Vec<OsString>,
}

impl From<Cli> for GitCtx {
    fn from(opts: Cli) -> Self {
        let Cli { cmd_set, .. } = opts;
        let cmd_set = match cmd_set {
            CommandSet::Git(opts) => opts,
            _ => unreachable!("This should not happen. The command is not a git shortcut"),
        };
        
        Self { repo: cmd_set[0].clone(), git_args: cmd_set[1..].to_vec() }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SharedCtx {
    pub run_hook: HookAction,
}

impl From<SharedOptions> for SharedCtx {
    fn from(opts: SharedOptions) -> Self {
        Self { run_hook: opts.run_hook }
    }
}

/// Behavior types for hook execution in shareable `--run-hook` flag.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum HookAction {
    /// Always execute hooks no questions asked.
    Always,

    /// Prompt user with hook's contents, and execute it if and only if user accepts it.
    #[default]
    Prompt,

    /// Never execute hooks no questions asked.
    Never,
}

/// Behavior types for list command.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ListAction {
    /// List full collection of repositories.
    #[default]
    All,

    /// Only list repositories that have been deployed.
    Deployed,

    /// Only list repositories that have not been deployed.
    Undeployed,
}

/// Fixup actions for `--fixup` flag in commit command.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum FixupAction {
    /// Amend changes in latest commit to all repositories.
    #[default]
    Amend,

    /// Fix text of latest commit to all repositories.
    Reword,
}
