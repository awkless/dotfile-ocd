// SPDX-FileCopyrightText: 2o24 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use mkdirp::mkdirp;
use snafu::{prelude::*, Whatever};
use std::{
    collections::HashMap,
    fs::{metadata, read_to_string, set_permissions, write},
    path::{Path, PathBuf},
};
use tempfile::{Builder as TempFileBuilder, TempDir};

/// Harness to manage multiple file fixtures in temporary directory.
pub struct FixtureHarness {
    root: TempDir,
    fixtures: HashMap<PathBuf, FileFixture>,
}

impl FixtureHarness {
    /// Open new fixture harness in new temporary directory.
    ///
    /// # Errors
    ///
    /// Will fail if temporary directory cannot be created for whatever reason.
    pub fn open() -> Result<Self, Whatever> {
        let root = TempFileBuilder::new()
            .tempdir()
            .with_whatever_context(|_| "Failed to construct temporary directory")?;
        Ok(Self { root, fixtures: HashMap::new() })
    }

    /// Add new [`FileFixture`] into harness.
    ///
    /// # Errors
    ///
    /// Will fail if file fixture cannot be created for whatever reason.
    pub fn with_file(
        mut self,
        path: impl AsRef<Path>,
        callback: impl FnOnce(FileFixtureBuilder) -> Result<FileFixture, Whatever>,
    ) -> Result<Self, Whatever> {
        let fixture = callback(FileFixture::builder(self.root.path().join(path.as_ref())))?;
        self.fixtures.insert(fixture.as_path().into(), fixture);
        Ok(self)
    }

    /// Get file fixture from harness.
    ///
    /// # Errors
    ///
    /// Will fail if file fixture is not being tracked by harness.
    pub fn get(&self, path: impl AsRef<Path>) -> Result<&FileFixture, Whatever> {
        self.fixtures
            .get(&self.root.path().join(path.as_ref()))
            .whatever_context(format!("Fixture '{}' is not being tracked", path.as_ref().display()))
    }

    /// Get mutable file fixture from harness.
    ///
    /// # Errors
    ///
    /// Will fail if file fixture is not being tracked by harness.
    pub fn get_mut(&mut self, path: impl AsRef<Path>) -> Result<&mut FileFixture, Whatever> {
        self.fixtures
            .get_mut(&self.root.path().join(path.as_ref()))
            .whatever_context(format!("Fixture '{}' is not being tracked", path.as_ref().display()))
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.root.path()
    }
}

/// File fixture handler.
///
/// Provides reliable way to create and maintain a file fixture for tests that
/// require file system interaction.
#[derive(Debug, Default, Clone)]
pub struct FileFixture {
    path: PathBuf,
    data: String,
    kind: FileKind,
}

impl FileFixture {
    /// Build new file fixture at target `path`.
    pub fn builder(path: impl Into<PathBuf>) -> FileFixtureBuilder {
        FileFixtureBuilder::new(path)
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Coerces to a [`str`] slice.
    pub fn as_str(&self) -> &str {
        self.data.as_ref()
    }

    /// Determine if file fixture has execute permission set.
    pub fn is_executable(&self) -> bool {
        self.kind == FileKind::Script
    }

    /// Syncronize file fixture.
    ///
    /// Ensure that file fixture remains in sync with file system.
    ///
    /// # Errors
    ///
    /// May fail if path to file fixture in file system cannot be read for
    /// whatever reason.
    pub fn sync(&mut self) -> Result<(), Whatever> {
        self.data = read_to_string(&self.path).with_whatever_context(|_| {
            format!("Failed to sync file fixture '{}'", self.path.display())
        })?;
        Ok(())
    }
}

/// Builder for [`FileFixture`].
#[derive(Debug, Default, Clone)]
pub struct FileFixtureBuilder {
    path: PathBuf,
    data: String,
    kind: FileKind,
}

impl FileFixtureBuilder {
    /// Construct empty file fixture at target `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), ..Default::default() }
    }

    /// Set initial data for file fixture.
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
    }

    /// Set the kind of file fixture.
    pub fn kind(mut self, kind: FileKind) -> Self {
        self.kind = kind;
        self
    }

    /// Write file fixture to file system.
    ///
    /// Will construct parent path if needed.
    ///
    /// # Errors
    ///
    /// May fail if parent path cannot be created, file fixture cannot be
    /// written at target path, or if execute permission cannot be set for
    /// whatever reason.
    pub fn write(self) -> Result<FileFixture, Whatever> {
        mkdirp(self.path.parent().unwrap())
            .with_whatever_context(|_| "Failed to create parent directory")?;
        write(&self.path, &self.data)
            .with_whatever_context(|_| "Failed to write file fixture data")?;

        #[cfg(unix)]
        if self.kind == FileKind::Script {
            use std::os::unix::fs::PermissionsExt;

            let metadata = metadata(&self.path)
                .with_whatever_context(|_| "Failed to get file fixture metadata")?;
            let mut perms = metadata.permissions();
            let mode = perms.mode();
            perms.set_mode(mode | 0o111);
            set_permissions(&self.path, perms)
                .with_whatever_context(|_| "Failed to give file fixture execute permission")?;
        }

        Ok(FileFixture { path: self.path, data: self.data, kind: self.kind })
    }
}

/// Select file fixture kind.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FileKind {
    /// Normal readable and writable file fixture.
    #[default]
    Normal,

    /// Readable and writable file fixture with execute permission.
    Script,
}
