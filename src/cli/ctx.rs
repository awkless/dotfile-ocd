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
