// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

mod ctx;

#[doc(inline)]
pub use ctx::*;

use clap::{Args, Error as ClapError, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use indoc::indoc;
use snafu::prelude::*;
use std::{ffi::OsString, path::PathBuf};

macro_rules! explain_cmd_shortcuts {
    () => {
        indoc! {r#"
        Command Shortcuts:
          <REPO> <GIT_CMD>  Shortcut to run user's Git binary on a target repository
        "#}
    };
}

#[derive(Debug, Parser)]
#[command(
    about,
    after_help = explain_cmd_shortcuts!(),
    after_long_help = explain_cmd_shortcuts!(),
    long_about = None,
    subcommand_help_heading = "Command Set",
    version,
    term_width = 80
)]
pub struct Cli {
    #[command(flatten, next_help_heading = "Logging Options")]
    pub log_opts: Verbosity<InfoLevel>,

    #[command(flatten)]
    pub shared_opts: SharedOptions,

    #[command(subcommand)]
    pub cmd_set: CommandSet,
}

impl Cli {
    /// Parse a set of command-line arguments.
    ///
    /// # Errors
    ///
    /// Will fail if given invalid arguments to parse.
    pub fn parse_args(
        args: impl IntoIterator<Item = impl Into<OsString> + Clone>,
    ) -> Result<Self, CliError> {
        let cli = Self::try_parse_from(args).context(BadParseSnafu)?;
        Ok(cli)
    }
}

#[derive(Debug, Subcommand)]
pub enum CommandSet {
    /// Initialize new repository.
    Init(InitOptions),

    /// Clone new repository.
    Clone(CloneOptions),

    /// Remove repository from collection.
    Remove(RemoveOptions),

    /// Deploy target repositories.
    Deploy(DeployOptions),

    /// Undeploy target repositories.
    Undeploy(UndeployOptions),

    /// List current set of repositories.
    List(ListOptions),

    /// Show status of repositories.
    Status(StatusOptions),

    /// Pull changes to all repositories.
    Pull(PullOptions),

    /// Push changes from all repositories.
    Push(PushOptions),

    /// Commit changes to all repositories.
    Commit(CommitOptions),

    /// Run user's Git binary on target repository.
    #[command(external_subcommand)]
    Git(Vec<OsString>),
}

#[derive(Args, Debug)]
pub struct InitOptions {
    pub name: String,

    #[arg(short = 'a', long, value_name = "DIR")]
    pub bare_alias: Option<PathBuf>,

    #[arg(short, long, value_name = "BRANCH")]
    pub branch: Option<String>,
}

#[derive(Args, Debug)]
pub struct CloneOptions {
    /// Remove to clone from.
    pub remote: String,

    /// Set name of cloned repository.
    pub repo: Option<String>,
}

#[derive(Args, Debug)]
pub struct RemoveOptions {
    /// Remove to clone from.
    pub remote: String,
}

#[derive(Args, Debug)]
pub struct DeployOptions {
    #[arg(num_args = 1.., value_delimiter = ',')]
    pub repos: Vec<String>,
}

#[derive(Args, Debug)]
pub struct UndeployOptions {
    #[arg(num_args = 1.., value_delimiter = ',')]
    pub repos: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ListOptions {
    #[arg(default_value_t = ListAction::default(), long, short, value_enum, value_name = "ACTION")]
    pub show: ListAction,
}

#[derive(Args, Debug)]
pub struct StatusOptions {
    /// Give a short status report.
    #[arg(long, short)]
    pub terse: bool,
}

#[derive(Args, Debug)]
pub struct PullOptions {
    /// Target remote to push to.
    pub remote: Option<String>,

    /// Target branch to push to.
    pub branch: Option<String>,
}

#[derive(Args, Debug)]
pub struct PushOptions {
    /// Target remote to push to.
    pub remote: Option<String>,

    /// Target branch to push to.
    pub branch: Option<String>,
}

#[derive(Args, Debug)]
pub struct CommitOptions {
    /// Amend or reword current commit.
    #[arg(long, short, value_name = "ACTION", value_enum)]
    pub fixup: Option<FixupAction>,

    /// Use MSG as the commit messsage.
    #[arg(long, short, value_name = "MSG")]
    pub message: Option<String>,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Command Options")]
pub struct SharedOptions {
    #[arg(default_value_t = HookAction::default(), long, short, value_enum, value_name = "ACTION")]
    pub run_hook: HookAction,
}

#[derive(Debug, Snafu)]
pub struct CliError(InnerCliError);

pub type Result<T, E = CliError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum InnerCliError {
    #[snafu(display("Failed to parse CLI arguments"))]
    BadParse { source: ClapError },
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::CommandFactory;
    use rstest::rstest;

    #[rstest]
    fn cli_verify_structure() {
        Cli::command().debug_assert();
    }
}
