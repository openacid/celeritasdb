use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

#[cfg(test)]
#[path = "./tests/exec_tests.rs"]
mod tests;

trait Edge<N> {
    fn target(&self) -> N;
}

impl Edge<usize> for usize {
    fn target(&self) -> usize {
        *self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    New,
    Repeated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Visit<N> {
    Edge { src: N, dst: N, status: Status },
    Retreat { u: N, parent: Option<N> },
    Root(N),
}

impl<T: fmt::Display> fmt::Display for Visit<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Visit::Edge { src, dst, status } => write!(f, "e({}-{})", src, dst),
            Visit::Retreat { u, parent } => write!(f, "r({})", u),
            Visit::Root(r) => write!(f, "R({})", r),
        }
    }
}

trait Graph {
    type Node: Copy + Eq + Hash;
    type Edge: Copy + Eq + Edge<Self::Node>;

    fn nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Node> + 'a>;

    fn edges<'a>(&'a self, u: &Self::Node) -> Box<dyn Iterator<Item = Self::Edge> + 'a>;

    fn dfs<'a>(&'a self) -> Dfs<'a, Self> {
        Dfs::new(self)
    }

    fn find_sccs(&self) -> Vec<Vec<Self::Node>> {
        Tarjan::from_graph(self).find_sccs()
    }
}

struct StackFrame<'a, G: Graph + ?Sized> {
    n: G::Node,
    neighbors: Box<dyn Iterator<Item = G::Edge> + 'a>,
}

impl<'a, G: Graph + ?Sized> StackFrame<'a, G> {
    fn new(g: &'a G, n: G::Node) -> StackFrame<'a, G> {
        StackFrame {
            n,
            neighbors: g.edges(&n),
        }
    }
}

struct Dfs<'a, G: Graph + ?Sized> {
    g: &'a G,
    visited: HashSet<G::Node>,
    stack: Vec<StackFrame<'a, G>>,
    roots: Box<dyn Iterator<Item = G::Node> + 'a>,
}

impl<'a, G: Graph + ?Sized> Dfs<'a, G> {
    fn new(g: &'a G) -> Dfs<'a, G> {
        Dfs {
            g,
            visited: HashSet::new(),
            stack: Vec::new(),
            roots: g.nodes(),
        }
    }

    fn next_root(&mut self) -> Option<G::Node> {
        while let Some(root) = self.roots.next() {
            if !self.visited.contains(&root) {
                return Some(root);
            }
        }
        None
    }

    fn cur_node(&self) -> Option<G::Node> {
        self.stack.last().map(|frame| frame.n)
    }
}

impl<'a, G: Graph + ?Sized> Iterator for Dfs<'a, G> {
    type Item = Visit<G::Node>;

    fn next(&mut self) -> Option<Visit<G::Node>> {
        if let Some(frame) = self.stack.last_mut() {
            let cur = frame.n;
            if let Some(next) = frame.neighbors.next() {
                let next = next.target();
                let st = if self.visited.contains(&next) {
                    Status::Repeated
                } else {
                    self.stack.push(StackFrame::new(self.g, next));
                    self.visited.insert(next);
                    Status::New
                };
                Some(Visit::Edge {
                    src: cur,
                    dst: next,
                    status: st,
                })
            } else {
                self.stack.pop();
                Some(Visit::Retreat {
                    u: cur,
                    parent: self.cur_node(),
                })
            }
        } else if let Some(next_root) = self.next_root() {
            self.stack.push(StackFrame::new(self.g, next_root));
            self.visited.insert(next_root);
            Some(Visit::Root(next_root))
        } else {
            None
        }
    }
}

struct NodeState {
    on_stack: bool,
    index: i32,
    lowlink: i32,
}

impl NodeState {
    fn new(idx: i32) -> NodeState {
        NodeState {
            on_stack: true,
            index: idx,
            lowlink: idx,
        }
    }
}

struct Tarjan<'a, G: Graph + ?Sized> {
    dfs: Dfs<'a, G>,
    stack: Vec<G::Node>,
    node_states: HashMap<G::Node, NodeState>,
    next_index: i32,
}

impl<'a, G: Graph + ?Sized> Tarjan<'a, G> {
    fn from_graph(g: &'a G) -> Self {
        Tarjan {
            dfs: g.dfs(),
            stack: Vec::new(),
            node_states: HashMap::new(),
            next_index: 0,
        }
    }

    fn find_sccs(mut self) -> Vec<Vec<G::Node>> {
        let mut ret = Vec::new();

        for visit in self.dfs {
            match visit {
                Visit::Retreat { u, parent } => {
                    let lowlink = self.node_states[&u].lowlink;
                    let index = self.node_states[&u].index;

                    if let Some(p) = parent {
                        self.node_states
                            .entry(p)
                            .and_modify(|s| s.lowlink = min(s.lowlink, lowlink));
                    }

                    if lowlink == index {
                        let mut scc = Vec::new();
                        loop {
                            let v = self.stack.pop().unwrap();
                            self.node_states.entry(v).and_modify(|s| s.on_stack = false);
                            scc.push(v.clone());
                            if v == u {
                                break;
                            }
                        }
                        ret.push(scc);
                    }
                }
                Visit::Root(u) => {
                    self.stack.push(u.clone());
                    self.node_states.insert(u, NodeState::new(self.next_index));
                    self.next_index += 1;
                }
                Visit::Edge { src, dst, status } => {
                    if status == Status::New {
                        self.stack.push(dst.clone());
                        self.node_states
                            .insert(dst, NodeState::new(self.next_index));
                        self.next_index += 1;
                    } else if self.node_states[&dst].on_stack {
                        // dst is on the stack implies that there is a path from dst to src
                        let index = self.node_states[&dst].index;
                        self.node_states
                            .entry(src)
                            .and_modify(|s| s.lowlink = min(s.lowlink, index));
                    }
                }
            }
        }

        ret
    }
}
