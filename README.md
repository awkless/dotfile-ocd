<!--
SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
SPDX-License-Identifier: MIT
-->

# Dotfile OCD

An experimental dotfile management CLI tool that allows the user to treat their
home directory like a git repository. The goal of this tool is to provide the
user a way to distribute their custom dotfile configurations without the need to
copy, move, or symlink them. The user can modularize their configurations via
multiple git repositories that can all be managed through the `dotfile-ocd`
binary.

## Requirements

Need the following software:

- [Git][git-scm] [>= 2.25.0].
- [Rust][rust-lang] [>= 1.77.2].

Git is needed, because the project relies on it to implement its core
functionality. Rust is required, because the project uses it as the primary
programming language along with Cargo to obtain its dependencies.

## Installation

The `dotfile-ocd` project is available through \<<https://crates.io>\> as a
lib+bin crate. Thus, anyone can obtain a functioning release through Cargo like
so:

```
# cargo install dotfile-ocd
```

The above method of installation will only provide the latest release published
to \<<https://crates.io>\>. However, if the latest changes to the project are
desired, then build through a clone of the project repository:

```
# git clone https://github.com/awkless/dotfile-ocd.git
# cd dotfile-ocd
# cargo build
# cargo install
```

It is recommended to install release versions of the project rather than
directly installing the latest changes made to the project repository. The clone
and build method previously shown should generally be used by those who intend
to contribute to the project.

## Usage

The `dotfile-ocd` project produces a binary named `ocd` for use at the command
line. This binary comes with following general structure:

```
# ocd [OPTIONS] <COMMAND>
```

__TODO:__ talk about command set (still need to figure that out).

## Contributing

The `dotfile-ocd` coding project is open to the following forms of contribution:

1. Improvements or additions to production code.
1. Improvements or additions to test code.
1. Improvements or additions to build system.
1. Improvements or additions to documentation.
1. Improvements or additions to CI/CD pipelines.

See the [contribution guidelines][contrib-guide] for more information about
contributing to the project.

## Copyright and Licensing

The `dotfile-ocd` project uses the MIT license as its main license for its
source code and documentation. The project also uses the CC0-1.0 license to
place files in the public domain that are considered to be to small or generic
to place copyright over.

This project uses the [REUSE 3.3 specification][reuse-3.3-spec] to make it
easier to determine who owns the copyright and licensing of any given file in
the codebase with SPDX identifiers. The [Developer Certificate of Origin version
1.1][linux-dco] to ensure that any conributions made have the right to be merged
into the project, and can be distributed with the project under its main
license.

[git-scm]: https://git-scm.com/downloads
[rust-lang]: https://www.rust-lang.org/learn/get-started
[reuse-3.3-spec]: https://reuse.software/spec-3.3/
[linux-dco]: https://developercertificate.org/
[contrib-guide]: CONTRIBUTING.md
