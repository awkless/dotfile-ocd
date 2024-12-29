// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use clap::{Args, Parser, Subcommand, Error as ClapError};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use snafu::prelude::*;

#[derive(Debug, Parser)]
#[command(
    about,
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

#[derive(Debug, Subcommand)]
pub enum CommandSet { }

#[derive(Debug, Args)]
#[command(next_help_heading = "Command Options")]
pub struct SharedOptions { }

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
