use petgraph::{algo, prelude::*};

#[test]
fn main() {
    let mut graph = DiGraph::<&str, i32>::new();

    let a = graph.add_node("a");
    let b = graph.add_node("b");
    let c = graph.add_node("c");
    let d = graph.add_node("d");

    graph.extend_with_edges(&[(a, b, 1), (b, c, 1), (c, d, 1), (a, b, 1), (b, d, 1)]);

    let ways = algo::all_simple_paths::<Vec<_>, _>(&graph, a, d, 0, None).collect::<Vec<_>>();

    println!("ok");
    assert_eq!(4, ways.len());
    for mut path in ways {
        println!("{:?}", path);
        assert_eq!(Some(d), path.pop())
    }
    assert!(false)
}
