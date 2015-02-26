#![allow(unstable)]
use std::io::{File, BufferedReader};
use graph::LabeledGraph;

mod graph;

static NO_PATH: &'static str = "No path found";
static WRONG_NODE_COUNT: &'static str = "You must provide a start and end node";

#[cfg(not(test))]
fn main() {
    use std::{io, os};
    use std::io::stdio::{StdWriter, StdinReader};

    let args = os::args();
    let graph_file = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None => panic!("Must provide graph data file")
    };
    let mut file_reader = open_file(graph_file);
    let graph = build_graph(&mut file_reader);
    let mut stdin = BufferedReader::new(io::stdin());
    let mut stdout = io::stdout();
    query_user(&mut stdout, &mut stdin, &graph);
}

/// Open the file as given by filename in the form of a Buffered Reader
/// #[cfg(not(test)]
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}

/// Create the graph by reading edges from the Buffered
fn build_graph<B: Buffer>(reader: &mut B) -> graph::LabeledGraph {
    let mut g = graph::LabeledGraph::new();
    for line in reader.lines() {
        let l: String  = line.unwrap();
        let node: &str = l.words().take(1).next().unwrap();
        let mut edges = l.words().skip(1);
        for neighbor in edges {
            g.add_edge(node, neighbor);
        }
    }
    g
}

/// Query the user to find out what shortest path they want to find
fn query_user<W: Writer, R: Buffer>(output: &mut W, input: &mut R,
                                    graph: &LabeledGraph) {
    loop {
        output.write_str("-> ");
        output.flush();
        let mut line = input.read_line().ok().unwrap();
        let nodes: Vec<&str> = line.words().collect();
        if nodes.len() == 2 {
            match graph.find_shortest_path(nodes[0], nodes[1]) {
                Some(v) => {
                    for n in v.iter() {
                        output.write_str(format!("{} ", n).as_slice());
                    }
                    output.write_str("\n");
                },
                None => {
                    output.write_line(NO_PATH);
                }
            }
        } else {
            output.write_line(WRONG_NODE_COUNT);
        }
    }
}

#[cfg(test)]
mod query_user_test {
    use super::query_user;
    use std::io::{MemReader, BufferedReader};

    #[test]
    fn test_query_user() {
    }

    fn run_test(input: &str, expected: HashMap<String, usize>) {
        let bytes = input.to_string().into_bytes();
        let r: BufferedReader<MemReader> =
            BufferedReader::new(MemReader::new(bytes));
        assert_eq!(train(r), expected);
    }
}
