<!--
SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
SPDX-License-Identifier: MIT
-->

# Contributing

Thanks for taking the time to contribute! Remember that the information stored
in this document only provides basic _guidelines_. Thus, all contributors are
expected to use their bast judgement!

## Expected Forms of Contributing

This project is open to the following forms of contribution:

1. Improvements or additions to production code.
1. Improvements or additions to test code.
1. Improvements or additions to build system.
1. Improvements or additions to documentation.
1. Improvements or additions to CI/CD pipelines.

Use the provided templates for bug reports, feature requests, and pull requests.
Please only use the bug tracker for reporting bug reports, or feature request
submissions.

Pull request submissions must occur on a separate branch, and compared by the
current state of the `main` branch. Keep commit history linear. Linear commit
history makes it easier to perform rebase merging. This project does not like
merge commits.

## Coding Style

The `dotfile-ocd` project uses the [Rust][rust-lang] programming langauge. Rust
already comes with a general [style and coding standard][rust-style] that should
be followed. To make development easier, use the `rustfmt` tool to automaically
format any piece of code, use `clippy` to lint your code, and use `cargo test`
to activate unit and integration testing to see if your code does not break
anything in the codebase.

## Commit Style

All commits in `dotfile-ocd` must abide by the [Conventional Commits
1.0.0][cc1.0.0] specification. Here are the following valid types for commits
accepted by this project:

- __chore__: General change that does not affect production code.
- __feat__: Change that implements a feature in production code.
- __fix__: Change that fixes a bug in production code.
- __docs__: Improvements or fixes to documentation.
- __style__: Improvements or fixes to the formatting of code.
- __refactor__: Changes involve refactoring the code base.
- __revert__: A set of commits were revereted.
- __test__: Test functionality of code base.
- __perf__: Improvements to performance of code base.

Try keep the subject, body, and trailer of your commits below 80 characters. Try
to keep commit history linear. Make sure that large changes get represented as a
series of commits rather than one massive commit. Make sure that your commits
are clear and descriptive.

Finally, The `dotfile-ocd` project uses the [Developer Certificate of Origin
version 1.1][linux-dco]. All commits need to have the following trailer:

```
Signed-off-by: <name> <email>
```

## Rules of Licensing and Copyright

This project abides by the [REUSE 3.3][reuse3] specification to determine the
licensing and copyright of files in the code base. Thus, all files must have the
proper SPDX copyright and licensing tags at the top always. Contributors can Use
the [reuse tool][reuse-tool] to determine if their changes are REUSE 3.3
compliant.

`dotfile-ocd` uses the MIT license as its main source code and documentation
license. `dotfile-ocd` also uses the CC0-1.0 license to place files in the
public domain that are considered to be to small or generic to place copyright
over. Thus, for almost all contributions you will use the MIT license.

Do not forget to include the following SPDX copyright identifier at the top of
any file you create along with the SPDX copyright identifier:

```
SPDX-FileCopyrightText: <year> <name> <email>
```

[rust-lang]: https://doc.rust-lang.org
[rust-style]: https://doc.rust-lang.org/beta/style-guide/index.html
[cc1.0.0]: https://www.conventionalcommits.org/en/v1.0.0/
[linux-dco]: https://en.wikipedia.org/wiki/Developer_Certificate_of_Origin
[reuse3]: https://reuse.software/spec/
[reuse-tool]: https://reuse.software/tutorial/
