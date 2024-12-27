// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

/// Handle repository dependencies.
///
/// Each repository configuration can have a bootstrap section that can
/// optionally come with a `depends` field. This special field defines a
/// set of other repositories that have been defined as dependencies that need
/// to be deployed or undeployed along with the given repository itself.
///
/// This type handles dependency information as a DAG (Directed Acyclic Graph).
/// This type also can handle the edge case where a repository does not have
/// a `depends` field.
#[derive(Debug)]
pub struct Dependencies {
    adj_list: HashMap<String, Vec<String>>,
}

impl Dependencies {
    pub fn new() -> Self {
        todo!();
    }

    pub fn add_vertex(&mut self, vertex: impl Into<String>) {
        todo!();
    }

    pub fn add_edge(&mut self, vertex: impl Into<String>, edge: impl Into<String>) {
        todo!();
    }

    pub fn topological_sort(&self) -> Option<Vec<String>> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
