// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::config::{ConfigFile, RepoConfig, Locator, RepoSettings};

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
    /// Construct new dependency handler.
    pub fn new() -> Self {
        Self { adj_list: HashMap::new() }
    }

    /// Load configuration file dependencies.
    pub fn with_config_file(&mut self, config: &ConfigFile<'_, RepoConfig, impl Locator>) {
        for repo in config.iter() {
            self.add_vertex(repo.name.clone());
            let deps = repo.bootstrap.as_ref().and_then(|b| b.depends.clone()).unwrap_or_default();
            for dep in deps {
                self.add_edge(repo.name.clone(), dep.clone());
            }
        }
    }

    /// Add new vertex.
    pub fn add_vertex(&mut self, vertex: String) {
        self.adj_list.entry(vertex.clone()).or_default();
    }

    /// Add new edge to given vertex.
    pub fn add_edge(&mut self, vertex: String, edge: String) {
        self.adj_list.entry(vertex.clone()).or_default().push(edge.clone());
        self.adj_list.entry(edge.clone()).or_default();
    }

    pub fn topological_sort(&self) -> Option<Vec<String>> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
