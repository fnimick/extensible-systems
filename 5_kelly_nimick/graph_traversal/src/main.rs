#![allow(unstable)]
use std::io::{File, BufferedReader};

mod graph;

#[cfg(not(test))]
fn main() {
    use std::os;
    use std::io;
    use std::io::stdio::StdinReader;

    let args = os::args();
    let graph_file = match args.iter().skip(1).take(1).next() {
        Some(file) => file.as_slice(),
        None => panic!("Must provide graph data file")
    };
    let mut file_reader = open_file(graph_file);
    let graph = build_graph(&mut file_reader);
}

/// Open the file as given by filename in the form of a Buffered Reader
fn open_file(filename: &str) -> BufferedReader<File> {
    let file = File::open(&Path::new(filename));
    BufferedReader::new(file.ok().expect("couldn't open file"))
}

fn build_graph<T: Reader>(reader: &mut BufferedReader<T>) -> graph::LabeledGraph {
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
