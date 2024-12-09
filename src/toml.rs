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
        let (key, value) = entry;
        info!("Add TOML entry '{}' to '{}' table", key.get(), table.as_ref());

        let entry = match self.get_table_mut(table.as_ref()) {
            Ok(table) => table,
            Err(InnerTomlError::TableNotFound { .. }) => {
                let mut new_table = Table::new();
                new_table.set_implicit(true);
                self.doc.insert(table.as_ref(), Item::Table(new_table));
                self.doc[table.as_ref()].as_table_mut().unwrap()
            }
            Err(err) => return Err(err.into()),
        };
        let entry = entry.insert(key.get(), value).map(|old| (key, old));

        Ok(entry)
    }

    pub fn remove(
        &mut self,
        table: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<(Key, Item), TomlError> {
        let entry = self.get_table_mut(table.as_ref())?;
        let entry = entry
            .remove_entry(key.as_ref())
            .context(EntryNotFoundSnafu { table: table.as_ref(), key: key.as_ref() })?;

        Ok(entry)
    }

    fn get_table(&self, key: &str) -> Result<&Table, InnerTomlError> {
        let table = self.doc.get(key).context(TableNotFoundSnafu { table: key })?;
        let table = table.as_table().context(NotTableSnafu { table: key })?;

        Ok(table)
    }

    fn get_table_mut(&mut self, key: &str) -> Result<&mut Table, InnerTomlError> {
        let table = self.doc.get_mut(key).context(TableNotFoundSnafu { table: key })?;
        let table = table.as_table_mut().context(NotTableSnafu { table: key })?;

        Ok(table)
    }
}

impl Display for Toml {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.doc)
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
pub struct TomlError(InnerTomlError);

#[derive(Debug, Snafu, PartialEq, Eq)]
enum InnerTomlError {
    #[snafu(display("Failed to parse TOML data"))]
    BadParse { source: TomlEditError },

    #[snafu(display("TOML table '{table}' not found"))]
    TableNotFound { table: String },

    #[snafu(display("TOML table '{table}' not defined as a table"))]
    NotTable { table: String },

    #[snafu(display("TOML entry '{key}' not found in table '{table}'"))]
    EntryNotFound { table: String, key: String },
}

pub type Result<T, E = TomlError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::{formatdoc, indoc};
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use snafu::report;
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
        assert!(matches!(result.unwrap_err().0, InnerTomlError::BadParse { .. }));
    }

    #[report]
    #[rstest]
    #[case("test", "foo", (Key::new("foo"), Item::Value(Value::from("hello"))))]
    #[case("test", "bar", (Key::new("bar"), Item::Value(Value::from(true))))]
    fn toml_get_return_key_item(
        toml_input: String,
        #[case] table: &str,
        #[case] key: &str,
        #[case] expect: (Key, Item),
    ) -> Result<()> {
        let toml: Toml = toml_input.parse()?;
        let (result_key, result_value) = toml.get(table, key)?;
        let (expect_key, expect_value) = expect;
        assert_eq!(result_key, &expect_key);
        assert_eq!(result_value.is_value(), expect_value.is_value());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::table_not_found(
        "bar = 'foo not here'",
        InnerTomlError::TableNotFound { table: "foo".into() },
    )]
    #[case::not_table(
        "foo = 'not a table'",
        InnerTomlError::NotTable { table: "foo".into() },
    )]
    #[case::entry_not_found(
        "[foo] # bar not here",
        InnerTomlError::EntryNotFound { table: "foo".into(), key: "bar".into() },
    )]
    fn toml_get_return_err(
        #[case] input: &str,
        #[case] expect: InnerTomlError,
    ) -> Result<()> {
        let toml: Toml = input.parse()?;
        let result = toml.get("foo", "bar");
        assert_eq!(result.unwrap_err().0, expect);

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::add_into_table(
        toml_input(),
        "test",
        (Key::new("baz"), Item::Value(Value::from("add this"))),
        formatdoc! {r#"
            {}baz = "add this"
        "#, toml_input()},
    )]
    #[case::create_new_table(
        toml_input(),
        "new_test",
        (Key::new("baz"), Item::Value(Value::from("add this"))),
        formatdoc! {r#"
            {}
            [new_test]
            baz = "add this"
        "#, toml_input()}
    )]
    fn toml_add_return_none(
        #[case] input: String,
        #[case] table: &str,
        #[case] entry: (Key, Item),
        #[case] expect: String,
    ) -> Result<()> {
        let mut toml: Toml = input.parse()?;
        let result = toml.add(table, entry)?;
        assert_eq!(toml.to_string(), expect);
        assert!(result.is_none());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::replace_foo(
        toml_input(),
        "test",
        (Key::new("foo"), Item::Value(Value::from("replaced"))),
        toml_input().replace(r#"foo = "hello""#, r#"foo = "replaced""#)
    )]
    #[case::replace_bar(
        toml_input(),
        "test",
        (Key::new("bar"), Item::Value(Value::from(false))),
        toml_input().replace(r#"bar = true"#, r#"bar = false"#)
    )]
    fn toml_add_return_some_key_item(
        #[case] input: String,
        #[case] table: &str,
        #[case] entry: (Key, Item),
        #[case] expect: String,
    ) -> Result<()> {
        let mut toml: Toml = input.parse()?;
        let result = toml.add(table, entry)?;
        assert_eq!(toml.to_string(), expect);
        assert!(result.is_some());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::not_table("foo = 'not a table'", InnerTomlError::NotTable { table: "foo".into() })]
    fn toml_add_return_err(
        #[case] input: &str,
        #[case] expect: InnerTomlError,
    ) -> Result<()> {
        let mut toml: Toml = input.parse()?;
        let stub = (Key::new("fail"), Item::Value(Value::from("this")));
        let result = toml.add("foo", stub);
        assert_eq!(result.unwrap_err().0, expect);

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::remove_foo(
        toml_input(),
        "test",
        "foo",
        (Key::new("foo"), Item::Value(Value::from("world"))),
        toml_input().replace("foo = \"hello\"\n", ""),
    )]
    #[case::remove_bar(
        toml_input(),
        "test",
        "bar",
        (Key::new("bar"), Item::Value(Value::from(true))),
        toml_input().replace("bar = true\n", ""),
    )]
    fn toml_remove_return_deleted_key_item(
        #[case] input: String,
        #[case] table: &str,
        #[case] key: &str,
        #[case] expect: (Key, Item),
        #[case] output: String,
    ) -> Result<()> {
        let mut toml: Toml = input.parse()?;
        let (return_key, return_value) = toml.remove(table, key)?;
        let (expect_key, expect_value) = expect;
        assert_eq!(toml.to_string(), output);
        assert_eq!(return_key, expect_key);
        assert_eq!(return_value.is_value(), expect_value.is_value());

        Ok(())
    }

    #[report]
    #[rstest]
    #[case::table_not_found(
        "bar = 'foo not here'",
        InnerTomlError::TableNotFound { table: "foo".into() },
    )]
    #[case::not_table(
        "foo = 'not a table'",
        InnerTomlError::NotTable { table: "foo".into() },
    )]
    #[case::entry_not_found(
        "[foo] # bar not here",
        InnerTomlError::EntryNotFound { table: "foo".into(), key: "bar".into() },
    )]
    fn toml_remove_return_err(
        #[case] input: &str,
        #[case] expect: InnerTomlError,
    ) -> Result<()> {
        let toml: Toml = input.parse()?;
        let result = toml.get("foo", "bar");
        assert_eq!(result.unwrap_err().0, expect);

        Ok(())
    }
}
