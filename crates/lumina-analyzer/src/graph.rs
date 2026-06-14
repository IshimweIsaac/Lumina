use std::collections::{HashMap, VecDeque};

pub type NodeId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldNode {
    pub entity: String,
    pub field: String,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub node_ids: HashMap<FieldNode, NodeId>,
    pub nodes: Vec<FieldNode>,
    pub dependents: Vec<Vec<NodeId>>,
    pub dependencies: Vec<Vec<NodeId>>,
    pub topo_order: Vec<NodeId>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            node_ids: HashMap::new(),
            nodes: Vec::new(),
            dependents: Vec::new(),
            dependencies: Vec::new(),
            topo_order: Vec::new(),
        }
    }

    pub fn intern(&mut self, entity: &str, field: &str) -> NodeId {
        let node = FieldNode {
            entity: entity.to_string(),
            field: field.to_string(),
        };
        if let Some(&id) = self.node_ids.get(&node) {
            id
        } else {
            let id = self.nodes.len() as u32;
            self.node_ids.insert(node.clone(), id);
            self.nodes.push(node);
            self.dependents.push(Vec::new());
            self.dependencies.push(Vec::new());
            id
        }
    }

    pub fn add_edge(&mut self, dep: NodeId, dependent: NodeId) {
        if !self.dependents[dep as usize].contains(&dependent) {
            self.dependents[dep as usize].push(dependent);
            self.dependencies[dependent as usize].push(dep);
        }
    }

    pub fn get_node(&self, entity: &str, field: &str) -> Option<NodeId> {
        self.node_ids
            .get(&FieldNode {
                entity: entity.to_string(),
                field: field.to_string(),
            })
            .copied()
    }

    pub fn compute_topo_order(&mut self) -> Result<(), CycleError> {
        let n = self.nodes.len();
        let mut in_degree = vec![0; n];
        for deps in &self.dependents {
            for &dest in deps {
                in_degree[dest as usize] += 1;
            }
        }

        let mut queue = VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i as u32);
            }
        }

        let mut order = Vec::new();
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &self.dependents[u as usize] {
                in_degree[v as usize] -= 1;
                if in_degree[v as usize] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if order.len() == n {
            self.topo_order = order;
            Ok(())
        } else {
            // Find cycle
            let mut chain = Vec::new();
            // Simple cycle detection for the error message
            // We just report the nodes that weren't visited
            for i in 0..n {
                if in_degree[i] > 0 {
                    chain.push(format!("{}.{}", self.nodes[i].entity, self.nodes[i].field));
                }
            }
            Err(CycleError { chain })
        }
    }

    pub fn recomputation_order(&self, changed: NodeId) -> Vec<NodeId> {
        let mut visited = vec![false; self.nodes.len()];
        let mut result = Vec::new();
        let mut stack = vec![changed];

        while let Some(u) = stack.pop() {
            if !visited[u as usize] {
                visited[u as usize] = true;
                result.push(u);
                for &v in &self.dependents[u as usize] {
                    stack.push(v);
                }
            }
        }

        // Sort by precomputed topo order
        let topo_map: HashMap<NodeId, usize> = self
            .topo_order
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();
        result.sort_by_key(|id| topo_map.get(id).unwrap_or(&usize::MAX));
        result
    }
}

#[derive(Debug)]
pub struct CycleError {
    pub chain: Vec<String>,
}

// ── v2.1.4: Resource dependency graph for cross-provider ordering ────────

#[derive(Debug, Clone)]
pub struct ResourceDependencyGraph {
    /// Map resource name → index
    indices: HashMap<String, usize>,
    /// Map index → resource name
    names: Vec<String>,
    /// Adjacency list: edges[i] = list of resources that i depends on
    edges: Vec<Vec<usize>>,
}

impl ResourceDependencyGraph {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            names: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn register(&mut self, name: &str) {
        if !self.indices.contains_key(name) {
            let idx = self.names.len();
            self.indices.insert(name.to_string(), idx);
            self.names.push(name.to_string());
            self.edges.push(Vec::new());
        }
    }

    /// Add a dependency edge: `dependent` depends on `dependency`
    pub fn add_dependency(&mut self, dependent: &str, dependency: &str) {
        self.register(dependent);
        self.register(dependency);
        let dep_idx = self.indices[dependency];
        let dependent_idx = self.indices[dependent];
        if !self.edges[dependent_idx].contains(&dep_idx) {
            self.edges[dependent_idx].push(dep_idx);
        }
    }

    /// Returns provision order (topological: dependencies first)
    pub fn provision_order(&self) -> Result<Vec<String>, CycleError> {
        self.topo_sort(false)
    }

    /// Returns destroy order (reverse topological: dependents first)
    pub fn destroy_order(&self) -> Result<Vec<String>, CycleError> {
        self.topo_sort(true)
    }

    /// Get the ordered list of dependencies for a single resource (including itself)
    pub fn provision_chain(&self, resource: &str) -> Result<Vec<String>, CycleError> {
        let full_order = self.provision_order()?;
        if let Some(&target_idx) = self.indices.get(resource) {
            // BFS to find all transitive dependencies
            let mut needed = vec![false; self.names.len()];
            let mut queue = VecDeque::new();
            needed[target_idx] = true;
            queue.push_back(target_idx);
            while let Some(u) = queue.pop_front() {
                for &dep in &self.edges[u] {
                    if !needed[dep] {
                        needed[dep] = true;
                        queue.push_back(dep);
                    }
                }
            }
            Ok(full_order.into_iter().filter(|name| {
                self.indices.get(name).map(|&i| needed[i]).unwrap_or(false)
            }).collect())
        } else {
            Ok(vec![resource.to_string()])
        }
    }

    fn topo_sort(&self, reverse: bool) -> Result<Vec<String>, CycleError> {
        let n = self.names.len();
        if n == 0 {
            return Ok(vec![]);
        }

        // Build in-degree based on edges (edges[i] = things i depends on)
        // For topo sort, we want to process dependencies first.
        // Build a forward graph: dependency → dependent
        let mut forward: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut in_degree = vec![0usize; n];
        for (dependent, deps) in self.edges.iter().enumerate() {
            for &dep in deps {
                forward[dep].push(dependent);
                in_degree[dependent] += 1;
            }
        }

        let mut queue = VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }

        let mut order = Vec::new();
        while let Some(u) = queue.pop_front() {
            order.push(self.names[u].clone());
            for &v in &forward[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if order.len() != n {
            let chain: Vec<String> = (0..n)
                .filter(|&i| in_degree[i] > 0)
                .map(|i| self.names[i].clone())
                .collect();
            return Err(CycleError { chain });
        }

        if reverse {
            order.reverse();
        }
        Ok(order)
    }
}
