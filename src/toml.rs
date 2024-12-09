// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use log::{info, trace};
use snafu::prelude::*;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use toml_edit::{DocumentMut, Item, Key, Table, TomlError as TomlEditError};

#[derive(Clone, Default, Debug)]
pub struct Toml {
    doc: DocumentMut,
}

impl Toml {
    pub fn new() -> Self {
        trace!("Construct new TOML parser");
        Self { doc: DocumentMut::new() }
    }

    pub fn get(
        &self,
        table: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<(&Key, &Item), TomlError> {
        info!("Get TOML entry '{}' from '{}' table", key.as_ref(), table.as_ref());
        let entry = self.get_table(table.as_ref())?;
        let entry = entry
            .get_key_value(key.as_ref())
            .context(EntryNotFoundSnafu { table: table.as_ref(), key: key.as_ref() })?;

        Ok(entry)
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

    pub(crate) fn get_table(&self, key: &str) -> Result<&Table, TomlError> {
        let table = self.doc.get(key).context(TableNotFoundSnafu { table: key })?;
        let table = table.as_table().context(NotTableSnafu { table: key })?;

        Ok(table)
    }
}

impl FromStr for Toml {
    type Err = TomlError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let doc: DocumentMut = data.parse().context(BadParseSnafu)?;
        Ok(Self { doc })
    }
}

#[derive(Debug, Snafu, PartialEq, Eq)]
pub enum TomlError {
    #[snafu(display("Failed to parse TOML data"))]
    BadParse { source: TomlEditError },

    #[snafu(display("TOML table '{table}' not found"))]
    TableNotFound { table: String },

    #[snafu(display("TOML table '{table}' not defined as a table"))]
    NotTable { table: String },

    #[snafu(display("TOML entry '{key}' not found in table '{table}'"))]
    EntryNotFound { table: String, key: String },
}

type Result<T, E = TomlError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::Report;
    use toml_edit::Value;

    #[fixture]
    fn toml_input() -> String {
        String::from(indoc! {r#"
            # this comment should remain!
            [test]
            foo = "hello"
            bar = true
        "#})
    }

    #[rstest]
    fn toml_parse_return_self(
        #[values("this = 'will parse'", "[so_will_this]", "hello = 'from ocd!'")] input: &str,
    ) {
        let toml: Result<Toml, TomlError> = input.parse();
        assert!(toml.is_ok());
    }

    #[rstest]
    fn toml_parse_return_err_bad_parse(
        #[values("this 'will fail'", "[will # also fail", "not.gonna = [work]")] input: &str,
    ) {
        let result: Result<Toml, TomlError> = input.parse();
        assert!(matches!(result.unwrap_err(), TomlError::BadParse { .. }));
    }

    #[rstest]
    #[case("test", "foo", (Key::new("foo"), Item::Value(Value::from("hello"))))]
    #[case("test", "bar", (Key::new("bar"), Item::Value(Value::from(true))))]
    fn toml_get_return_key_item(
        toml_input: String,
        #[case] table: &str,
        #[case] key: &str,
        #[case] expect: (Key, Item),
    ) -> Result<(), Report<TomlError>> {
        let toml: Toml = toml_input.parse().map_err(Report::from_error)?;
        let (result_key, result_value) = toml.get(table, key).map_err(Report::from_error)?;
        let (expect_key, expect_value) = expect;
        assert_eq!(result_key, &expect_key);
        assert_eq!(result_value.is_value(), expect_value.is_value());

        Ok(())
    }

    #[rstest]
    #[case::table_not_found(
        "bar = 'foo not here'",
        TomlError::TableNotFound { table: "foo".into() },
    )]
    #[case::not_table(
        "foo = 'not a table'",
        TomlError::NotTable { table: "foo".into() },
    )]
    #[case::entry_not_found(
        "[foo] # bar not here",
        TomlError::EntryNotFound { table: "foo".into(), key: "bar".into() },
    )]
    fn toml_get_return_err(
        #[case] input: &str,
        #[case] expect: TomlError,
    ) -> Result<(), Report<TomlError>> {
        let toml: Toml = input.parse().map_err(Report::from_error)?;
        let result = toml.get("foo", "bar");
        assert_eq!(result.unwrap_err(), expect);

        Ok(())
    }
}
