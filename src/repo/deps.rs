// SPDX-FileCopyrightText: 2024 Jason Pena <jasonpena@awkless.com>
// SPDX-License-Identifier: MIT

use crate::config::{ConfigFile, Locator, RepoConfig};

use snafu::prelude::*;
use std::collections::{HashMap, VecDeque, HashSet};

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
    pub fn add_vertex(&mut self, vertex: impl Into<String>) {
        self.adj_list.entry(vertex.into()).or_default();
    }

    /// Add new edge to given vertex.
    pub fn add_edge(&mut self, vertex: impl Into<String>, edge: impl Into<String>) {
        let edge = edge.into();
        self.adj_list.entry(vertex.into()).or_default().push(edge.clone());
        self.adj_list.entry(edge).or_default();
    }

    /// Check that no dependencies are circular.
    pub fn acyclic_check(&self) -> Result<(), DependencyError> {
        let result = self.topological_sort();
        if result.len() != self.adj_list.len() {
            return Err(DependencyError(InnerDependencyError::FoundCycle {
                deps: result.join(" "),
            }));
        }

        Ok(())
    }

    /// Produce topological sort of dependencies.
    pub fn topological_sort(&self) -> Vec<String> {
        // Use Kahn's algorithm for topological sorting...
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        // Initialize in-degree for each node...
        for edges in self.adj_list.values() {
            for edge in edges {
                *in_degree.entry(edge.clone()).or_insert(0) += 1;
            }
        }

        // Add vertex with in-degree 0 to queue...
        for vertex in self.adj_list.keys() {
            if !in_degree.contains_key(vertex) {
                queue.push_back(vertex.clone());
            }
        }

        // Perform BFS...
        while let Some(vertex) = queue.pop_front() {
            result.push(vertex.clone());
            if let Some(edges) = self.adj_list.get(&vertex) {
                for edge in edges {
                    *in_degree.get_mut(edge).unwrap() -= 1;
                    if *in_degree.get(edge).unwrap() == 0 {
                        queue.push_back(edge.clone());
                    }
                }
            }
        }

        result
    }
}

#[derive(Debug, Snafu)]
pub struct DependencyError(InnerDependencyError);

pub type Result<T, E = DependencyError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum InnerDependencyError {
    #[snafu(display("Following repositories defined as circular dependencies: '{deps}'"))]
    FoundCycle { deps: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    fn dependencies_acyclic_check_return_err() {
        let mut deps = Dependencies::new();
        deps.add_vertex("vim");
        deps.add_vertex("foo");
        deps.add_vertex("bar");
        deps.add_edge("vim", "bar");
        deps.add_edge("bar", "foo");
        deps.add_edge("foo", "vim");
        let result = deps.acyclic_check();
        assert!(matches!(result.unwrap_err().0, InnerDependencyError::FoundCycle { .. }));
    }

    #[rstest]
    fn dependencies_acyclic_check_return_ok() {
        let mut deps = Dependencies::new();
        deps.add_vertex("vim");
        deps.add_vertex("foo");
        deps.add_vertex("bar");
        deps.add_edge("vim", "bar");
        deps.add_edge("bar", "foo");
        let result = deps.acyclic_check();
        assert!(result.is_ok());
    }
}
