// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

mod cli;
mod config;
mod repo;

#[cfg(test)]
mod testenv;

use crate::{
    cli::{Cli, CliError},
    config::ConfigError,
    repo::RepoManagerError,
};

use env_logger::Builder as EnvLogBuilder;
use log::{error, LevelFilter};
use snafu::{prelude::*, Report};
use std::{env::args_os, ffi::OsString, process::exit};

fn main() {
    EnvLogBuilder::new()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(LevelFilter::max())
        .format_indent(Some(8))
        .init();

    let code = match run(args_os) {
        Ok(code) => code,
        Err(err) => {
            let err = Report::from_error(err);
            error!("{err}");
            ExitCode::Failure
        }
    }
    .into();

    exit(code);
}

fn run<I, F>(args: F) -> Result<ExitCode, BinError>
where
    I: IntoIterator<Item = OsString>,
    F: FnOnce() -> I + Clone,
{
    let opts = Cli::parse_args(args()).context(CliSnafu)?;
    log::set_max_level(opts.log_opts.log_level_filter());

    Ok(ExitCode::Success)
}

#[derive(Debug)]
enum ExitCode {
    Success,
    Failure,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        match code {
            ExitCode::Success => 0,
            ExitCode::Failure => 1,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum BinError {
    #[snafu(display("dotfile-ocd cli failure"))]
    Cli { source: CliError },

    #[snafu(display("dotfile-ocd configuration file failure"))]
    ConfigFile { source: ConfigError },

    #[snafu(display("dotfile-ocd repository manager failure"))]
    RepoManager { source: RepoManagerError },
}
