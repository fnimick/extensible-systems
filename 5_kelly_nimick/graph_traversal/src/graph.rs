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
#[derive(Show)]
struct Graph {
    edges: Vec<BitvSet>,
}

impl Graph {
    fn new() -> Graph {
        Graph{
            edges: Vec::new(),
        }
    }

    // Adds a node and returns its index
    fn add_node(&mut self) -> usize {
        self.edges.push(BitvSet::new());
        self.edges.len() - 1
    }

    // Edge addition
    fn add_edge(&mut self, source: usize, target: usize) {
        // checks to make sure that these nodes exist
        assert!(source < self.edges.len());
        assert!(target < self.edges.len());
        self.edges[source].insert(target);
        self.edges[target].insert(source);
    }

    // Uses Dijkstra's algorithm to find the shortest path from the
    // source to the target node
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
    #[test]
    fn test_graph() {
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
        assert_eq!(g.find_shortest_path(3, 2).unwrap().len(), 2);
        assert_eq!(g.find_shortest_path(0, 3).unwrap().len(), 3);
    }
}

pub struct LabeledGraph<'a> {
    labels: HashMap<String, usize>,
    indices: Vec<String>,
    graph: Graph,
}


impl<'a> LabeledGraph<'a> {
    pub fn new() -> Self {
        LabeledGraph {
            labels: HashMap::new(),
            indices: Vec::new(),
            graph: Graph::new(),
        }
    }

    // If the node label does not exist, adds it to the graph
    // If it does exist, does nothing
    fn add_node_if_not_exists(&mut self, node: &str) {
        let key = String::from_str(node);
        if self.labels.contains_key(&key) { return; }
        let index = self.graph.add_node();
        self.labels.insert(key.clone(), index);
        self.indices.push(key);
    }

    // Adds an edge from source label to target label
    // Adds the associated nodes if they do not already exist
    pub fn add_edge(&mut self, source: &str, target: &str) {
        self.add_node_if_not_exists(source);
        self.add_node_if_not_exists(target);
        let (s, t) = (source.to_string(), target.to_string());
        let source_idx = *self.labels.get(&s).unwrap();
        let target_idx = *self.labels.get(&t).unwrap();
        self.graph.add_edge(source_idx, target_idx);
    }

    // Finds the shortest path in a LabeledGraph
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
    use super::LabeledGraph;
    #[test]
    fn test_labeled_graph() {
        let mut g = LabeledGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        g.add_edge("c", "d");
        assert_eq!(g.find_shortest_path("a", "b").unwrap().len(), 2);
        assert_eq!(g.find_shortest_path("b", "c").unwrap().len(), 2);
        assert_eq!(g.find_shortest_path("a", "c").unwrap().len(), 3);
        assert_eq!(g.find_shortest_path("c", "a").unwrap().len(), 3);
        assert_eq!(g.find_shortest_path("d", "a").unwrap().len(), 4);
        assert_eq!(g.find_shortest_path("a", "d").unwrap(),
                   vec!["a", "b", "c", "d"]);
    }
}
