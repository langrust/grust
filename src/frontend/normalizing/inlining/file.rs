use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::hir::{
    equation::Equation, file::File, identifier_creator::IdentifierCreator,
    stream_expression::StreamExpression,
};

impl File {
    fn remove_shifted_causality_loop(&mut self, graphs: &mut HashMap<String, Graph<Color>>) {
        let unitary_nodes_to_visit = self
            .nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    node.unitary_nodes
                        .keys()
                        .map(|output_id| output_id.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        unitary_nodes_to_visit
            .iter()
            .for_each(|(node_id, output_ids)| {
                output_ids.iter().for_each(|output_id| {
                    self.remove_shifted_causality_loop_visit(&node_id, output_id, graphs)
                });
            });
    }

    fn remove_shifted_causality_loop_visit(
        &mut self,
        node_id: &String,
        output_id: &String,
        graphs: &mut HashMap<String, Graph<Color>>,
    ) {
        let mut nodes = self
            .nodes
            .iter()
            .map(|node| (&node.id, node))
            .collect::<HashMap<_, _>>();

        let node = nodes.get(node_id).unwrap();
        let graph = graphs.get_mut(node_id).unwrap();

        // construct output's subgraph
        let subgraph = graph.subgraph_from_vertex(&output_id);
        let useful_signals = subgraph.get_vertices();

        // create identifier creator contianing the useful signals
        let mut identifier_creator = IdentifierCreator::from(useful_signals.clone());

        // get useful equations
        let equations = useful_signals
            .iter()
            .filter_map(|signal| node.unscheduled_equations.get(signal))
            .map(|equation| equation.clone());

        // compute new equations and memory for the unitary node
        let mut new_equations: Vec<Equation> = vec![];
        equations.for_each(|equation| match &equation.expression {
            StreamExpression::NodeApplication {
                node,
                inputs,
                signal,
                ..
            } => {
                let should_inline = equation
                    .expression
                    .get_dependencies()
                    .iter()
                    .any(|(id, depth)| equation.id.eq(id) && *depth >= 1);

                // if one input depends directly on the output
                // then the node call must be inlined
                if should_inline {
                    let called_node = nodes.get(node).unwrap();
                    let called_unitary_node = called_node.unitary_nodes.get(signal).unwrap();

                    // get equations from called node, with corresponding inputs
                    let mut reduced_equations = called_unitary_node.instantiate_equations(
                        &mut identifier_creator,
                        inputs,
                        &equation.id,
                        &equation.scope,
                    );

                    // insert reduced equations
                    new_equations.append(&mut reduced_equations);
                } else {
                    // insert current equation
                    new_equations.push(equation.clone());
                }
            }
            _ => new_equations.push(equation.clone()),
        });

        // put new equations in node
        let node = nodes.get_mut(node_id).unwrap();
        todo!()
    }
}
