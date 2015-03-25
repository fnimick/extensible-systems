#[doc="
    Module: Graph

    This module contains the LabeledGraph code. This is a general structure which is used
    by the MBTA struture defined in the T module. It exposes operations such as
    find_shortest_path which allows an external client to find a path through the
    graph, as well as add_edge to create the graph structure.
"]


use std::collections::{BitvSet, BinaryHeap, HashMap};
use std::usize;
use std::cmp::Ordering;

// This is necessary for the min-priority queue used in Graph::find_shortest_path
#[derive(Eq, PartialEq, PartialOrd)]
struct State {
    cost: usize,
    position: usize,
    path: Vec<usize>,
}

// Flip the ordering so BinaryHeap finds mins, not maxes
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

// Represents an edge in the adjacency list
#[derive(Eq, PartialEq, PartialOrd, Show)]
struct Edge {
    node: usize,
    cost: usize,
}

// Graph in adjacency list representation
// edges[index] represents the adjacency list for node # index
// BitvSet is used to ensure that we don't create duplicate edges
#[derive(Show, Eq, PartialEq, PartialOrd)]
struct Graph {
    edges: Vec<Vec<Edge>>,
}

impl Graph {
    /// Create a new Graph structure
    fn new() -> Graph {
        Graph{
            edges: Vec::new(),
        }
    }

    /// Adds a node and returns its index
    fn add_node(&mut self) -> usize {
        self.edges.push(Vec::new());
        self.edges.len() - 1
    }

    /// Edge addition
    fn add_edge(&mut self, source: usize, target: usize, weight: Option<usize>) {
        let weight = weight.unwrap_or(1);
        // checks to make sure that these nodes exist
        assert!(source < self.edges.len());
        assert!(target < self.edges.len());
        self.edges[source].push(Edge { node: target, cost: weight });
        self.edges[target].push(Edge { node: source, cost: weight });
    }

    /// Uses Dijkstra's algorithm to find the shortest path from the
    /// source to the target node
    fn find_shortest_path(&self, source: usize, target: usize) -> Option<Vec<usize>> {
        // cost[node] is the cost of the shortest path from source to node,
        // and the path expressed as a Vec<usize>
        let mut cost: Vec<(usize, Vec<usize>)> = (0..self.edges.len())
            .map(|_| (usize::MAX, Vec::new())).collect();

        // we're currently at node `source`, zero distance
        cost[source] = (0, vec![source]);

        // create our min-priority queue
        let mut queue = BinaryHeap::new();
        queue.push(State { cost: 0, position: source, path: vec![source]});

        // while let: https://github.com/rust-lang/rfcs/pull/214
        while let Some(State { cost: current_cost, position, path }) = queue.pop() {
            // if we've already found a better way, skip and keep going
            if current_cost > cost[position].0 { continue; }

            // For each node reachable from our current position,
            // see if there exists a shorter path through our current position
            // than currently calculated for that node
            for &Edge { node, cost: edge_cost } in self.edges[position].iter() {
                let new_cost = current_cost + edge_cost;
                if new_cost < cost[node].0 {
                    // we've found a better way
                    let mut path_vec = path.clone();
                    path_vec.push(node);
                    cost[node] = (new_cost, path_vec.clone());
                    queue.push(State { cost: new_cost, position: node, path: path_vec });
                }
            }
        }

        let path_vec = &cost[target].1;
        if path_vec.is_empty() {
            None
        } else {
            Some(path_vec.clone())
        }
    }
}

#[cfg(test)]
mod graph_test {
    use super::Graph;
    use std::collections::BitvSet;
    use std::collections::bitv::Bitv;

    #[test]
    fn test_add_node() {
        let mut g = Graph::new();
        assert!(g.edges.is_empty());
        g.add_node();
        assert_eq!(g.edges.len(), 1);
    }

    //#[test]
    /*
    fn test_add_edge() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        assert!(g.edges[0].is_empty());
        assert!(g.edges[1].is_empty());
        g.add_edge(0, 1);
        assert_eq!(g.edges[0], BitvSet::from_bitv(Bitv::from_bytes(&[0b01000000])));
        assert_eq!(g.edges[1], BitvSet::from_bitv(Bitv::from_bytes(&[0b10000000])));
    }*/

    #[test]
    #[should_fail]
    fn test_add_invalid_edge() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        g.add_edge(1, 2, None);
    }

    #[test]
    fn test_shortest_path() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        g.add_node();
        g.add_node();
        g.add_edge(0, 1, None);
        g.add_edge(1, 2, None);
        g.add_edge(0, 2, Some(4));
        g.add_edge(2, 3, None);
        //assert_eq!(g.find_shortest_path(0, 1).unwrap().len(), 2);
        //assert_eq!(g.find_shortest_path(1, 2).unwrap().len(), 2);
        assert_eq!(g.find_shortest_path(0, 2).unwrap().len(), 3);
        //assert_eq!(g.find_shortest_path(0, 3).unwrap().len(), 3);
    }
}

#[derive(Show, Hash, Clone, Eq, PartialEq)]
pub struct Node {
    pub station: String,
    pub line: String
}

/// LabeledGraph is a wrapper around Graph that supports named
/// nodes.
#[derive(Show, Eq, PartialEq)]
pub struct LabeledGraph {
    labels: HashMap<Node, usize>,
    indices: Vec<Node>,
    graph: Graph,
}

impl LabeledGraph {
    /// Create a new LabeledGraph
    pub fn new() -> Self {
        LabeledGraph {
            labels: HashMap::new(),
            indices: Vec::new(),
            graph: Graph::new(),
        }
    }

    /// Add a node to the graph if it doesn't already exist
    fn add_node_if_not_exists(&mut self, key: &Node) {
        if self.labels.contains_key(key) { return; }
        let index = self.graph.add_node();
        self.labels.insert(key.clone(), index);
        self.indices.push(key.clone());
    }

    /// Adds an edge from source label to target label
    /// Adds the associated nodes if they do not already exist
    pub fn add_edge(&mut self, source: &Node, target: &Node, weight: Option<usize>) {
        self.add_node_if_not_exists(source);
        self.add_node_if_not_exists(target);
        let source_idx = *self.labels.get(source).unwrap();
        let target_idx = *self.labels.get(target).unwrap();
        self.graph.add_edge(source_idx, target_idx, weight);
    }

    /// Finds the shortest path in a LabeledGraph
    pub fn find_shortest_path(&self, source: &Node, target: &Node)
            -> Option<Vec<Node>> {
        if !self.labels.contains_key(source) ||
                !self.labels.contains_key(target) {
            return None;
        }
        let source_idx = *self.labels.get(source).unwrap();
        let target_idx = *self.labels.get(target).unwrap();
        match self.graph.find_shortest_path(source_idx, target_idx) {
            Some(result) => {
                Some(result.iter().map(|&: &n| {
                    self.indices[n].clone()
                }).collect())
            },
            None => None
        }
    }
}
