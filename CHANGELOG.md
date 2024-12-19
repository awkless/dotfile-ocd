<!--
SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
SPDX-License-Identifier: MIT
-->


# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### [0.2.0] - 2024-12-19

### Added

- Add `crate::config::Locator` to handle the tracking configuration path data.
    - Add `crate::config::XdgLocator` to handle configuration path data through
      the XDG Base Directory specification.
- Add `crate::config::Toml` to parse TOML file data.
- Add `crate::config::Settings` to perform serialization and deserialization of
  TOML file data.
    - Add `crate::config::RepoSettings` with `crate::config::BootstrapSettings`
      to serialize and deserialize repository configuration settings.
    - Add `crate::config::CmdHookSettings` with `HookSettings` to serialize and
      deserialize commnad hook configuration settings.
- Add `crate::config::ConfigFile` to handle configuration file management
  through a set of configuration strategies.
    - Add `crate::config::Config` trait to define standard interface for
      configuration file manipulation strategies.
    - Add `crate::config::RepoConfig` strategy to handle repository
      configuration file.
    - Add `crate::config::CmdHookConfig` strategy to handle command hook
      configuration file.
- Add `crate::testenv::FixtureHarness` to manage a collection of file fixtures
  during testing.
    - Add `crate::testenv::FileFixture` to help define and manage file fixtures
      for testing in code base.

### [0.1.0] - 2024-12-07

### Added

- Place project under MIT license.
- Add CC0-1.0 license to place some stuff in public domain.
- Add `README.md` file.
- Add `CONTRIBUTING.md` file.
- Setup Cargo build system.
- Define `main` for `ocd` binary.
- Add CI code quality check.
- Add CI REUSE v3.3 compliance check.
- Define default textual attributes in `.gitattributes`.
- Ignore auxiliary build files from Cargo.
- Provide pull request template.
- Provide bug report template.
- Provide feature request template.
- Make @awkless main code owner of project.

[Unreleased]: https://github.com/awkless/dotfile-ocd/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/awkless/dotfile-ocd/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/awkless/dotfile-ocd/releases/tag/v0.1.0
