// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

mod ctx;

#[doc(inline)]
pub use ctx::*;

use clap::{Args, Parser, Subcommand, Error as ClapError};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use snafu::prelude::*;
use std::ffi::OsString;

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
