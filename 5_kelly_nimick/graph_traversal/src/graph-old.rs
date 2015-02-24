pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

pub struct Node {
    first_edge: Option<usize>,
}

pub struct Edge {
    next_edge: [Option<usize>; 2],
    source: usize,
    target: usize,
}


impl Graph {
    pub fn new() -> Graph {
        Graph{
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    // Node addition
    pub fn next_node_index(&self) -> usize { self.nodes.len() }

    pub fn add_node(&mut self) -> usize {
        let index = self.next_node_index();
        self.nodes.push(Node {
            first_edge: None,
        });
        index
    }

    // Edge addition
    pub fn next_edge_index(&self) -> usize { self.edges.len() }

    pub fn add_edge(&mut self, source: usize, target: usize) -> usize {
        assert!(source < self.nodes.len());
        assert!(target < self.edges.len());
        let index = self.next_edge_index();

        // get the current head of the edges list from source and target nodes
        let source_edge = self.nodes[source].first_edge;
        let target_edge = self.nodes[target].first_edge;

        // add the new edge to the list of edges in the graph
        self.edges.push(Edge {
            next_edge: [source_edge, target_edge],
            source: source,
            target: target,
        });

        // modify the source and target nodes to point to this edge first
        self.nodes[source].first_edge = Some(index);
        self.nodes[target].first_edge = Some(index);
        index
    }

    pub fn find_shortest_path(&self, source: usize, target: usize)
        -> Option<Vec<usize>> {
        
}
