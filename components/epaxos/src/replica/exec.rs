use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

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
            Visit::Edge {
                src,
                dst,
                status: _,
            } => write!(f, "e({}-{})", src, dst),
            Visit::Retreat { u, parent: _ } => write!(f, "r({})", u),
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

#[cfg(test)]
mod tests {
    use super::Graph;
    use super::Status::*;
    use super::Visit::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct TestNode {
        next: Vec<usize>,
    }

    #[derive(Debug)]
    struct GraphData {
        nodes: HashMap<usize, TestNode>,
        keys: Vec<usize>,
    }

    impl Graph for GraphData {
        type Node = usize;
        type Edge = usize;

        fn nodes<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
            Box::new(self.keys.iter().cloned())
        }

        fn edges<'a>(&'a self, u: &usize) -> Box<dyn Iterator<Item = usize> + 'a> {
            Box::new(self.nodes[u].next.iter().cloned())
        }
    }

    // Given a string like "0-3, 1-2, 3-4, 2-3", creates a graph.
    fn graph(s: &str) -> GraphData {
        let mut ret = GraphData {
            nodes: HashMap::new(),
            keys: vec![],
        };

        for e in s.split(',') {
            let dash_idx = e.find('-').unwrap();
            let u: usize = e[..dash_idx].trim().parse().unwrap();
            let v: usize = e[(dash_idx + 1)..].trim().parse().unwrap();

            ret.nodes.entry(u).or_insert(TestNode { next: Vec::new() });
            ret.nodes.entry(v).or_insert(TestNode { next: Vec::new() });
            ret.nodes.get_mut(&u).unwrap().next.push(v);

            if let None = ret.keys.iter().find(|&&x| x == u) {
                ret.keys.push(u)
            }

            if let None = ret.keys.iter().find(|&&x| x == v) {
                ret.keys.push(v)
            }
            ret.keys.sort();
        }

        ret
    }

    #[test]
    fn test_dfs() {
        let cases = vec![
            (
                // visited order
                "0-1, 0-3, 0-2",
                "R(0),e(0-1),r(1),e(0-3),r(3),e(0-2),r(2),r(0)",
            ),
            (
                // repeat visit
                "0-1, 0-2, 1-2",
                "R(0),e(0-1),e(1-2),r(2),r(1),e(0-2),r(0)",
            ),
            (
                // multi root nodes
                "0-1, 2-3, 4-5",
                "R(0),e(0-1),r(1),r(0),R(2),e(2-3),r(3),r(2),R(4),e(4-5),r(5),r(4)",
            ),
            (
                // loop
                "0-1, 1-2, 2-0",
                "R(0),e(0-1),e(1-2),e(2-0),r(2),r(1),r(0)",
            ),
        ];
        for case in cases {
            let g = graph(case.0);
            let dfs: Vec<String> = g.dfs().map(|x| format!("{}", x)).collect();
            assert_eq!(dfs.join(","), case.1);
        }
    }
}
