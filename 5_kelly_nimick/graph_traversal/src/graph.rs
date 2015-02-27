use std::collections::{BitvSet, BinaryHeap, HashMap};
use std::usize;
use std::cmp::Ordering;

// This is necessary for the min-priority queue used in Graph::find_shortest_path
#[derive(Eq, PartialEq, PartialOrd)]
struct State {
    distance: usize,
    position: usize,
    path: Vec<usize>,
}

// Flip the ordering so BinaryHeap finds mins, not maxes
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

// Graph in adjacency list representation
// edges[index] represents the adjacency list for node # index
// BitvSet is used to ensure that we don't create duplicate edges
#[derive(Show, Eq, PartialEq, PartialOrd)]
struct Graph {
    edges: Vec<BitvSet>,
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
        self.edges.push(BitvSet::new());
        self.edges.len() - 1
    }

    /// Edge addition
    fn add_edge(&mut self, source: usize, target: usize) {
        // checks to make sure that these nodes exist
        assert!(source < self.edges.len());
        assert!(target < self.edges.len());
        self.edges[source].insert(target);
    }

    /// Uses Dijkstra's algorithm to find the shortest path from the
    /// source to the target node
    fn find_shortest_path(&self, source: usize, target: usize) -> Option<Vec<usize>> {
        // dist[node] is the length of the shortest path from source to node,
        // and the path expressed as a Vec<usize>
        let mut dist: Vec<(usize, Vec<usize>)> = (0..self.edges.len())
            .map(|_| (usize::MAX, Vec::new())).collect();

        // we're currently at node `source`, zero distance
        dist[source] = (0, vec![source]);

        // create our min-priority queue
        let mut queue = BinaryHeap::new();
        queue.push(State { distance: 0, position: source, path: vec![source]});

        // while let: https://github.com/rust-lang/rfcs/pull/214
        while let Some(State { distance, position, path }) = queue.pop() {
            if position == target { return Some(path); }

            // if we've already found a better way, skip and keep going
            if distance > dist[position].0 { continue; }

            // For each node reachable from our current position,
            // see if there exists a shorter path through our current position
            // than currently calculated for that node
            for edge in self.edges[position].iter() {
                let new_dist = distance + 1;
                if new_dist < dist[edge].0 {
                    // we've found a better way
                    let mut path_vec = path.clone();
                    path_vec.push(edge);
                    dist[edge] = (new_dist, path_vec.clone());
                    queue.push(State { distance: new_dist, position: edge, path: path_vec });
                }
            }
        }

        // no path exists from source to target
        None
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

    #[test]
    fn test_add_edge() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        assert!(g.edges[0].is_empty());
        assert!(g.edges[1].is_empty());
        g.add_edge(0, 1);
        assert_eq!(g.edges[0], BitvSet::from_bitv(Bitv::from_bytes(&[0b01000000])));
        assert!(g.edges[1].is_empty());
        g.add_edge(1, 0);
        assert_eq!(g.edges[0], BitvSet::from_bitv(Bitv::from_bytes(&[0b01000000])));
        assert_eq!(g.edges[1], BitvSet::from_bitv(Bitv::from_bytes(&[0b10000000])));
    }

    #[test]
    #[should_fail]
    fn test_add_invalid_edge() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        g.add_edge(1, 2);
    }

    #[test]
    fn test_shortest_path() {
        let mut g = Graph::new();
        g.add_node();
        g.add_node();
        g.add_node();
        g.add_node();
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(0, 2);
        g.add_edge(2, 3);
        assert_eq!(g.find_shortest_path(0, 1).unwrap().len(), 2);
        assert_eq!(g.find_shortest_path(1, 2).unwrap().len(), 2);
        assert_eq!(g.find_shortest_path(0, 2).unwrap().len(), 2);
        assert_eq!(g.find_shortest_path(3, 2), None);
        assert_eq!(g.find_shortest_path(0, 3).unwrap().len(), 3);
    }
}

/// LabeledGraph is a wrapper around Graph that supports named
/// nodes.
#[derive(Show, Eq, PartialEq)]
pub struct LabeledGraph {
    labels: HashMap<String, usize>,
    indices: Vec<String>,
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
    fn add_node_if_not_exists(&mut self, node: &str) {
        let key = String::from_str(node);
        if self.labels.contains_key(&key) { return; }
        let index = self.graph.add_node();
        self.labels.insert(key.clone(), index);
        self.indices.push(key);
    }

    /// Adds an edge from source label to target label
    /// Adds the associated nodes if they do not already exist
    pub fn add_edge(&mut self, source: &str, target: &str) {
        self.add_node_if_not_exists(source);
        self.add_node_if_not_exists(target);
        let (s, t) = (source.to_string(), target.to_string());
        let source_idx = *self.labels.get(&s).unwrap();
        let target_idx = *self.labels.get(&t).unwrap();
        self.graph.add_edge(source_idx, target_idx);
    }

    /// Finds the shortest path in a LabeledGraph
    pub fn find_shortest_path(&self, source_str: &str, target_str: &str)
            -> Option<Vec<&str>> {
        let (source, target) = (source_str.to_string(), target_str.to_string());
        if !self.labels.contains_key(&source) ||
                !self.labels.contains_key(&target) {
            return None;
        }
        let source_idx = *self.labels.get(&source).unwrap();
        let target_idx = *self.labels.get(&target).unwrap();
        match self.graph.find_shortest_path(source_idx, target_idx) {
            Some(result) => {
                Some(result.iter().map(|&: &n| {
                    self.indices[n].as_slice()
                }).collect())
            },
            None => None
        }
    }
}

#[cfg(test)]
mod labeled_graph_test {
    use super::{Graph, LabeledGraph};

    #[test]
    fn test_add_edge() {
        let mut lg = LabeledGraph::new();
        let mut g = Graph::new();
        assert!(lg.labels.is_empty());
        assert!(lg.indices.is_empty());
        assert_eq!(lg.graph, g);
        lg.add_edge("a", "b");
        assert_eq!(*lg.labels.get("a").unwrap(), 0);
        assert_eq!(*lg.labels.get("b").unwrap(), 1);
        assert_eq!(lg.indices, vec!["a", "b"]);
        g.add_node();
        g.add_node();
        g.add_edge(0, 1);
        assert_eq!(lg.graph, g);
        lg.add_edge("c", "b");
        assert_eq!(*lg.labels.get("a").unwrap(), 0);
        assert_eq!(*lg.labels.get("b").unwrap(), 1);
        assert_eq!(*lg.labels.get("c").unwrap(), 2);
        assert_eq!(lg.indices, vec!["a", "b", "c"]);
        g.add_node();
        g.add_edge(2, 1);
        assert_eq!(lg.graph, g);
    }

    #[test]
    fn test_shortest_path() {
        let mut g = LabeledGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        g.add_edge("c", "d");
        assert_eq!(g.find_shortest_path("a", "b").unwrap().len(), 2);
        assert_eq!(g.find_shortest_path("b", "c").unwrap().len(), 2);
        assert_eq!(g.find_shortest_path("a", "c").unwrap().len(), 3);
        assert_eq!(g.find_shortest_path("c", "a"), None);
        assert_eq!(g.find_shortest_path("d", "a"), None);
        assert_eq!(g.find_shortest_path("a", "d").unwrap(),
                   vec!["a", "b", "c", "d"]);
    }
}
