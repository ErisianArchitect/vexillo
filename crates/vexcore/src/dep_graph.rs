use std::collections::{HashMap, HashSet};

use syn::Ident;



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
}