// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

mod locate;
mod toml;

use env_logger::Builder as EnvLogBuilder;
use log::{error, LevelFilter};
use snafu::{ErrorCompat, Whatever};
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
            error!("{err}");
            if let Some(bt) = ErrorCompat::backtrace(&err) {
                error!("{bt}");
            }
            ExitCode::Failure
        }
    }
    .into();

    exit(code);
}

fn run<I, F>(_args: F) -> Result<ExitCode, Whatever>
where
    I: IntoIterator<Item = OsString>,
    F: FnOnce() -> I + Clone,
{
    todo!();
}

#[derive(Debug)]
pub enum ExitCode {
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
