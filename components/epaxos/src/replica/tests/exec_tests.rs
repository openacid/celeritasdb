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
fn test_graph() {
    let cases = vec![
        (
            // visited order
            // 0→1
            // ↓↘
            // 2  3
            "0-1, 0-3, 0-2",
            "R(0),e(0-1),r(1),e(0-3),r(3),e(0-2),r(2),r(0)",
            vec![vec![1usize], vec![3], vec![2], vec![0]],
        ),
        (
            // repeat visit
            // 0→1
            // ↓↙
            // 2
            "0-1, 0-2, 1-2",
            "R(0),e(0-1),e(1-2),r(2),r(1),e(0-2),r(0)",
            vec![vec![2usize], vec![1], vec![0]],
        ),
        (
            // multi root nodes
            // 0→1  2→3  4→5
            "0-1, 2-3, 4-5",
            "R(0),e(0-1),r(1),r(0),R(2),e(2-3),r(3),r(2),R(4),e(4-5),r(5),r(4)",
            vec![vec![1usize], vec![0], vec![3], vec![2], vec![5], vec![4]],
        ),
        (
            // loop
            // 0→1
            // ↑↙
            // 2
            "0-1, 1-2, 2-0",
            "R(0),e(0-1),e(1-2),e(2-0),r(2),r(1),r(0)",
            vec![vec![2usize, 1, 0]],
        ),
        (
            // loop connect loop
            // 0← 2-----→3↔ 4
            // ↓↗
            // 1
            "0-1, 1-2, 2-0, 2-3, 3-4, 4-3",
            "R(0),e(0-1),e(1-2),e(2-0),e(2-3),e(3-4),e(4-3),r(4),r(3),r(2),r(1),r(0)",
            vec![vec![4usize, 3], vec![2, 1, 0]],
        ),
    ];
    for case in cases {
        let g = graph(case.0);
        let dfs: Vec<String> = g.dfs().map(|x| format!("{}", x)).collect();
        assert_eq!(case.1, dfs.join(","));
        assert_eq!(case.2, g.find_sccs());
    }
}
