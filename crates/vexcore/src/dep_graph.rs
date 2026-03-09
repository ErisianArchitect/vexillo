use std::collections::{HashMap, HashSet, VecDeque};

use syn::Ident;


/// Dependency Graph Node.
/// Contains a set of dependencies, and a set of
/// dependents, representing edges in a dependency
/// graph.
#[derive(Default)]
struct DepNode<'a> {
    dependencies: HashSet<&'a Ident>,
    dependents: HashSet<&'a Ident>,
}

pub struct DepGraph<'a> {
    nodes: HashMap<&'a Ident, HashSet<&'a Ident>>,
}

impl<'a> DepGraph<'a> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
    
    pub fn insert<It: IntoIterator<Item = &'a Ident>>(&mut self, ident: &'a Ident, dependencies: It) {
        let node_mut = self.nodes.entry(ident).or_insert_with(HashSet::new);
        node_mut.extend(dependencies);
    }
    
    pub fn sort(&self) -> Result<Vec<&'a Ident>, ()> {
        // By this point, the dependency graph has only been built with
        // nodes that have their edges defined by dependencies, but not
        // dependents.
        // This topological sort algorithm depends on knowing a node's
        // dependencies AND dependents.
        // A new graph needs to be rebuilt, by stepping through the
        // pre-built graph and iterating over each node(N)'s dependencies,
        // and inserting node(n) into the dependency node's dependents set.
        let mut full_graph = HashMap::new();
        for (&key, dependencies) in self.nodes.iter() {
            full_graph.entry(key).or_insert_with(|| DepNode {
                dependencies: HashSet::with_capacity(dependencies.len()),
                dependents: HashSet::new(),
            }).dependencies.extend(dependencies.iter().copied());
            for dep in dependencies.iter().copied() {
                let dep_node = full_graph.entry(dep).or_insert_with(|| DepNode {
                    dependencies: HashSet::new(),
                    dependents: HashSet::new(),
                });
                dep_node.dependents.insert(key);
            }
        }
        let mut order = Vec::with_capacity(full_graph.len());
        let mut queue = VecDeque::with_capacity(full_graph.len());
        let mut in_degrees = HashMap::with_capacity(full_graph.len());
        let mut total_degrees = 0usize;
        for (&key, node) in full_graph.iter() {
            if node.dependencies.len() == 0 {
                queue.push_back(key);
            } else {
                let node_dep_count = node.dependencies.len();
                total_degrees += node_dep_count;
                in_degrees.insert(key, node_dep_count);
            }
        }
        
        while let Some(next_key) = queue.pop_front() {
            order.push(next_key);
            for dep_node in full_graph[next_key].dependents.iter().copied() {
                let Some(degrees) = in_degrees.get_mut(dep_node) else {
                    unreachable!("in_degrees should always be valid.");
                };
                *degrees -= 1;
                total_degrees -= 1;
                if *degrees == 0 {
                    queue.push_back(dep_node);
                }
            }
        }
        if total_degrees > 0 {
            return Err(());
        }
        Ok(order)
    }
}

#[test]
fn dep_graph_test() {
    let i: [Ident; 8] = [
        syn::parse_quote!(i0),
        syn::parse_quote!(i1),
        syn::parse_quote!(i2),
        syn::parse_quote!(i3),
        syn::parse_quote!(i4),
        syn::parse_quote!(i5),
        syn::parse_quote!(i6),
        syn::parse_quote!(i7),
    ];
    // macro_rules! name {
    //     ($name:ident) => {
    //         let $name: Ident = syn::parse_quote!($name);
    //     };
    // }
    let mut graph = DepGraph::new();
    graph.insert(&i[0], []);
    graph.insert(&i[1], []);
    graph.insert(&i[2], [&i[1]]);
    graph.insert(&i[3], [&i[2]]);
    graph.insert(&i[4], [&i[3]]);
    graph.insert(&i[5], [&i[4]]);
    graph.insert(&i[6], [&i[5]]);
    graph.insert(&i[7], [&i[6]]);
    if let Ok(sort) = graph.sort() {
        assert_eq!(sort.len(), 8);
        for node in sort {
            println!("{node}");
        }
    } else {
        panic!("Cycle detected.");
    }
}

/*
ident: additions: [ident], removals: [ident]
ident: mask

*/