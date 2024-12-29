// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use clap::ValueEnum;

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
