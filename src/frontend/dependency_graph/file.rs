use std::collections::HashMap;

use crate::error::Error;
use crate::hir::file::File;

impl File {
    /// Generate dependency graph for every nodes/component.
    pub fn generate_dependency_graphs(&self, errors: &mut Vec<Error>) -> Result<(), ()> {
        let File {
            nodes, component, ..
        } = self;

        // initialize dictionaries for graphs
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();

        // initialize every nodes' graphs
        nodes
            .into_iter()
            .map(|node| {
                let graph = node.create_initialized_graph();
                nodes_graphs.insert(node.id.clone(), graph.clone());
                nodes_reduced_graphs.insert(node.id.clone(), graph.clone());
                Ok(())
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // optional component's graph initialization
        component.as_ref().map_or(Ok(()), |component| {
            let graph = component.create_initialized_graph();
            nodes_graphs.insert(component.id.clone(), graph.clone());
            nodes_reduced_graphs.insert(component.id.clone(), graph.clone());
            Ok(())
        })?;

        // creates nodes context: nodes dictionary
        let nodes_context = nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect::<HashMap<_, _>>();

        // every nodes complete their dependency graphs
        nodes
            .into_iter()
            .map(|node| {
                node.add_all_dependencies(
                    &nodes_context,
                    &mut nodes_graphs,
                    &mut nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // optional component completes its dependency graph
        component.as_ref().map_or(Ok(()), |component| {
            component.add_all_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod generate_dependency_graphs {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::{
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, file::File, node::Node,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_generate_dependency_graphs_for_all_nodes() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            scope: Scope::Local,
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            scope: Scope::Input,
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        let file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![node],
            component: None,
            location: Location::default(),
        };

        file.generate_dependency_graphs(&mut errors).unwrap();

        let graph = file.nodes.get(0).unwrap().graph.get().unwrap();

        let mut control = Graph::new();
        control.add_vertex(String::from("o"), Color::Black);
        control.add_vertex(String::from("x"), Color::Black);
        control.add_vertex(String::from("i"), Color::Black);
        control.add_edge(&String::from("x"), String::from("i"), 0);
        control.add_edge(&String::from("o"), String::from("x"), 0);

        assert_eq!(*graph, control);
    }
}
