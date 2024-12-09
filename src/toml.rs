// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use snafu::prelude::*;
use std::str::FromStr;
use toml_edit::{DocumentMut, Item, Key, TomlError as TomlEditError};

#[derive(Clone, Default, Debug)]
pub struct Toml {
    doc: DocumentMut,
}

impl Toml {
    pub fn new() -> Self {
        todo!();
    }

    pub fn get(
        &self,
        table: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<(&Key, &Item), TomlError> {
        todo!();
    }

    pub fn add(
        &mut self,
        table: impl AsRef<str>,
        entry: (Key, Item),
    ) -> Result<Option<(Key, Item)>, TomlError> {
        todo!();
    }

    pub fn remove(
        &mut self,
        table: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<(Key, Item), TomlError> {
        todo!();
    }
}

impl FromStr for Toml {
    type Err = TomlError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let doc: DocumentMut = data.parse().context(BadParseSnafu)?;
        Ok(Self { doc })
    }
}

#[derive(Debug, Snafu)]
pub enum TomlError {
    #[snafu(display("Failed to parse TOML data"))]
    BadParse { source: TomlEditError },
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use snafu::Whatever;

    #[rstest]
    fn toml_parse_return_self(
        #[values("this = 'will parse'", "[so_will_this]", "hello = 'from ocd!'")] input: &str,
    ) -> Result<(), Whatever> {
        let toml: Result<Toml, TomlError> = input.parse();
        assert!(toml.is_ok());

        Ok(())
    }

    #[rstest]
    fn toml_parse_return_err_bad_parse(
        #[values("this 'will fail'", "[will # also fail", "not.gonna = [work]")] input: &str,
    ) -> Result<(), Whatever> {
        let result: Result<Toml, TomlError> = input.parse();
        assert!(matches!(result.unwrap_err(), TomlError::BadParse { .. }));

        Ok(())
    }
}
