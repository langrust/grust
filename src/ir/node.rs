use std::collections::HashMap;

use crate::common::{
    color::Color,
    graph::{neighbor::Neighbor, Graph},
    location::Location,
    scope::Scope,
    type_system::Type,
};
use crate::error::Error;
use crate::ir::equation::Equation;

use super::unitary_node::UnitaryNode;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's unscheduled equations.
    pub unscheduled_equations: HashMap<String, Equation>,
    /// Unitary output nodes generated from this node.
    pub unitary_nodes: HashMap<String, UnitaryNode>,
    /// Node location.
    pub location: Location,
}

impl Node {
    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, node::Node, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     color::Color, constant::Constant, graph::Graph, location::Location,
    ///     scope::Scope, type_system::Type,
    /// };
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    ///
    /// let graph = node.create_initialized_graph();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::White);
    /// control.add_vertex(String::from("x"), Color::White);
    /// control.add_vertex(String::from("i"), Color::White);
    ///
    /// assert_eq!(graph, control);
    /// ```
    pub fn create_initialized_graph(&self) -> Graph<Color> {
        // create an empty graph
        let mut graph = Graph::new();

        // get node's signals
        let Node {
            inputs,
            unscheduled_equations,
            ..
        } = self;

        // add input signals as vertices
        for (input, _) in inputs {
            graph.add_vertex(input.clone(), Color::White);
        }

        // add other signals as vertices
        for (signal, _) in unscheduled_equations {
            graph.add_vertex(signal.clone(), Color::White);
        }

        // return graph
        graph
    }

    /// Complete dependencies graph of the node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) { // depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    ///
    /// This example correspond to the following test.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, node::Node, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     color::Color, constant::Constant, graph::Graph, location::Location,
    ///     scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     node
    /// );
    /// let node = nodes_context.get(&String::from("test")).unwrap();
    ///
    /// let graph = node.create_initialized_graph();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph();
    /// let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);
    ///
    /// node.add_all_dependencies(&nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    ///
    /// let graph = nodes_graphs.get(&node.id).unwrap();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::Black);
    /// control.add_vertex(String::from("x"), Color::Black);
    /// control.add_vertex(String::from("i"), Color::Black);
    /// control.add_edge(&String::from("x"), String::from("i"), 0);
    /// control.add_edge(&String::from("o"), String::from("x"), 0);
    ///
    /// assert_eq!(*graph, control);
    /// ```
    pub fn add_all_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            inputs,
            unscheduled_equations,
            ..
        } = self;

        // add local and output signals dependencies
        unscheduled_equations
            .keys()
            .map(|signal| {
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // add input signals dependencies
        // (makes vertices colors "Black" => equal assertions in tests)
        inputs
            .iter()
            .map(|(signal, _)| {
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Add direct dependencies of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    ///
    /// This example correspond to the following test.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, node::Node, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     color::Color, constant::Constant, graph::Graph, location::Location,
    ///     scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     node
    /// );
    /// let node = nodes_context.get(&String::from("test")).unwrap();
    ///
    /// let graph = node.create_initialized_graph();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph();
    /// let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);
    ///
    /// node.add_signal_dependencies(&String::from("x"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    /// node.add_signal_dependencies(&String::from("o"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    /// node.add_signal_dependencies(&String::from("i"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    ///
    /// let graph = nodes_graphs.get(&node.id).unwrap();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::Black);
    /// control.add_vertex(String::from("x"), Color::Black);
    /// control.add_vertex(String::from("i"), Color::Black);
    /// control.add_edge(&String::from("x"), String::from("i"), 0);
    /// control.add_edge(&String::from("o"), String::from("x"), 0);
    ///
    /// assert_eq!(*graph, control);
    /// ```
    pub fn add_signal_dependencies(
        &self,
        signal: &String,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            id: node,
            unscheduled_equations,
            location,
            ..
        } = self;

        // get node's graph
        let graph = nodes_graphs.get_mut(node).unwrap();
        // get signal's vertex
        let vertex = graph.get_vertex_mut(signal);

        match vertex.get_value() {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                vertex.set_value(Color::Grey);

                unscheduled_equations
                    .get(signal)
                    .map_or(Ok(()), |equation| {
                        // retrieve expression
                        let expression = &equation.expression;

                        // get dependencies
                        let dependencies = expression.get_dependencies(
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        // get node's graph (borrow checker)
                        let graph = nodes_graphs.get_mut(node).unwrap();

                        // add dependencies as graph's edges:
                        // s = e depends on s' <=> s -> s'
                        dependencies
                            .iter()
                            .for_each(|(id, depth)| graph.add_edge(signal, id.clone(), *depth));

                        Ok(())
                    })?;

                // get node's graph (borrow checker)
                let graph = nodes_graphs.get_mut(node).unwrap();
                // get signal's vertes (borrow checker)
                let vertex = graph.get_vertex_mut(signal);
                // update status: processed
                vertex.set_value(Color::Black);

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let error = Error::NotCausal {
                    node: node.clone(),
                    signal: signal.clone(),
                    location: location.clone(),
                };
                errors.push(error);
                Err(())
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }

    /// Add dependencies to node's inputs of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x which depends on input i
    ///     x: int = i;     // depends on input i
    /// }
    /// ```
    ///
    /// This example correspond to the following test.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, node::Node, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     color::Color, constant::Constant, graph::Graph, location::Location,
    ///     scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     node
    /// );
    /// let node = nodes_context.get(&String::from("test")).unwrap();
    ///
    /// let graph = node.create_initialized_graph();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph();
    /// let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);
    ///
    /// node.add_signal_inputs_dependencies(&String::from("x"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    /// node.add_signal_inputs_dependencies(&String::from("o"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    /// node.add_signal_inputs_dependencies(&String::from("i"), &nodes_context, &mut nodes_graphs, &mut nodes_reduced_graphs, &mut errors).unwrap();
    ///
    /// let reduced_graph = nodes_reduced_graphs.get(&node.id).unwrap();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::Black);
    /// control.add_vertex(String::from("x"), Color::Black);
    /// control.add_vertex(String::from("i"), Color::Black);
    /// control.add_edge(&String::from("x"), String::from("i"), 0);
    /// control.add_edge(&String::from("o"), String::from("i"), 0);
    ///
    /// assert_eq!(*reduced_graph, control);
    /// ```
    pub fn add_signal_inputs_dependencies(
        &self,
        signal: &String,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            id: node, inputs, ..
        } = self;

        // get node's reduced graph
        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
        // get signal's vertex
        let reduced_vertex = reduced_graph.get_vertex_mut(signal);

        match reduced_vertex.get_value() {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                reduced_vertex.set_value(Color::Grey);

                // compute signals dependencies
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // get node's graph
                let graph = nodes_graphs.get_mut(node).unwrap();
                // get signal's vertex
                let vertex = graph.get_vertex_mut(signal);

                // for every neighbors, get inputs dependencies
                for Neighbor { id, weight: w1 } in vertex.get_neighbors() {
                    // tells if the neighbor is an input
                    let is_input = inputs.iter().any(|(input, _)| *input == id);

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        reduced_graph.add_edge(signal, id, w1);
                    } else {
                        // else compute neighbor's inputs dependencies
                        self.add_signal_inputs_dependencies(
                            &id,
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // get neighbor's vertex
                        let reduced_vertex = reduced_graph.get_vertex(&id);

                        // add dependencies as graph's edges:
                        // s = e depends on i <=> s -> i
                        reduced_vertex.get_neighbors().into_iter().for_each(
                            |Neighbor { id, weight: w2 }| {
                                reduced_graph.add_edge(signal, id, w1 + w2)
                            },
                        );
                    }
                }

                // get node's reduced graph (borrow checker)
                let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                // get signal's vertex (borrow checker)
                let reduced_vertex = reduced_graph.get_vertex_mut(signal);
                reduced_vertex.set_value(Color::Black);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Generate unitary nodes from mother node.
    ///
    /// Generate and add unitary nodes to mother node.
    /// Unitary nodes are nodes with one output and contains
    /// all signals from which the output computation depends.
    ///
    /// Unitary nodes computations induces schedulings of the node.
    /// This detects causality errors.
    ///
    /// It also detects unused signal definitions or inputs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::common::{
    ///     color::Color, graph::Graph, location::Location, scope::Scope, type_system::Type,
    /// };
    /// use grustine::ir::{
    ///     equation::Equation, node::Node, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let mut node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![
    ///         (String::from("i1"), Type::Integer),
    ///         (String::from("i2"), Type::Integer),
    ///     ],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o1"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o1"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///         (
    ///             String::from("o2"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o2"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i2"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    ///
    /// let mut graph = Graph::new();
    /// graph.add_vertex(String::from("i1"), Color::Black);
    /// graph.add_vertex(String::from("i2"), Color::Black);
    /// graph.add_vertex(String::from("x"), Color::Black);
    /// graph.add_vertex(String::from("o1"), Color::Black);
    /// graph.add_vertex(String::from("o2"), Color::Black);
    /// graph.add_edge(&String::from("x"), String::from("i1"), 0);
    /// graph.add_edge(&String::from("o1"), String::from("x"), 0);
    /// graph.add_edge(&String::from("o2"), String::from("i2"), 0);
    ///
    /// node.generate_unitary_nodes(&mut graph, &mut errors)
    ///     .unwrap();
    ///
    /// let unitary_node_1 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("o1"),
    ///     inputs: vec![(String::from("i1"), Type::Integer)],
    ///     scheduled_equations: vec![
    ///         Equation {
    ///             scope: Scope::Local,
    ///             id: String::from("x"),
    ///             signal_type: Type::Integer,
    ///             expression: StreamExpression::SignalCall {
    ///                 id: String::from("i1"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///         Equation {
    ///             scope: Scope::Output,
    ///             id: String::from("o1"),
    ///             signal_type: Type::Integer,
    ///             expression: StreamExpression::SignalCall {
    ///                 id: String::from("x"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let unitary_node_2 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("o2"),
    ///     inputs: vec![(String::from("i2"), Type::Integer)],
    ///     scheduled_equations: vec![
    ///         Equation {
    ///             scope: Scope::Output,
    ///             id: String::from("o2"),
    ///             signal_type: Type::Integer,
    ///             expression: StreamExpression::SignalCall {
    ///                 id: String::from("i2"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let control = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![
    ///         (String::from("i1"), Type::Integer),
    ///         (String::from("i2"), Type::Integer),
    ///     ],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o1"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o1"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///         (
    ///             String::from("o2"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o2"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i2"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::from([(String::from("o2"), unitary_node_2), (String::from("o1"), unitary_node_1)]),
    ///     location: Location::default(),
    /// };
    ///
    /// assert_eq!(node, control)
    /// ```
    pub fn generate_unitary_nodes(
        &mut self,
        graph: &mut Graph<Color>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        // get outputs identifiers
        let outputs = self
            .unscheduled_equations
            .values()
            .filter(|equation| equation.scope.eq(&Scope::Output))
            .map(|equation| equation.id.clone())
            .collect::<Vec<_>>();

        // construct unitary node for each output
        let subgraphs = outputs
            .into_iter()
            .map(|output| self.add_unitary_node(output, graph, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, ()>>()?;

        // check that every signals are used
        let unused_signals = graph.forgotten_vertices(subgraphs);
        unused_signals
            .into_iter()
            .map(|signal| {
                let error = Error::UnusedSignal {
                    node: self.id.clone(),
                    signal,
                    location: self.location.clone(),
                };
                errors.push(error);
                Err(())
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()
    }

    fn add_unitary_node(
        &mut self,
        output: String,
        graph: &mut Graph<Color>,
        errors: &mut Vec<Error>,
    ) -> Result<Graph<Color>, ()> {
        let Node {
            id: node,
            inputs,
            unscheduled_equations,
            unitary_nodes,
            location,
            ..
        } = self;

        // construct unitary node's subgraph from its output
        let mut subgraph = graph.subgraph_from_vertex(&output);

        // schedule the unitary node
        let schedule = subgraph.topological_sorting(errors).map_err(|signal| {
            let error = Error::NotCausal {
                node: node.clone(),
                signal: signal,
                location: location.clone(),
            };
            errors.push(error)
        })?;

        // get usefull inputs (in application order)
        let unitary_node_inputs = inputs
            .into_iter()
            .filter(|(id, _)| schedule.contains(id))
            .map(|input| input.clone())
            .collect::<Vec<_>>();

        // retrieve scheduled equations from schedule
        // and mother node's equations
        let scheduled_equations = schedule
            .into_iter()
            .filter_map(|signal| unscheduled_equations.get(&signal))
            .map(|equation| equation.clone())
            .collect();

        // construct unitary node
        let unitary_node = UnitaryNode {
            node_id: node.clone(),
            output_id: output.clone(),
            inputs: unitary_node_inputs,
            scheduled_equations,
            location: location.clone(),
        };

        // insert it in node's storage
        unitary_nodes.insert(output.clone(), unitary_node);

        Ok(subgraph)
    }

    /// Normalize IR node.
    ///
    /// Normalize all unitary nodes of a node as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are normalized into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::{HashSet, HashMap};
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
    ///
    /// let unitary_nodes_used_inputs = HashMap::from([
    ///     (
    ///         String::from("my_node"),
    ///         HashMap::from([(String::from("o"), vec![true, true])]),
    ///     ),
    ///     (
    ///         String::from("other_node"),
    ///         HashMap::from([(String::from("o"), vec![true, true])]),
    ///     )
    /// ]);
    ///
    /// let equation_1 = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("x"),
    ///     signal_type: Type::Integer,
    ///     expression: StreamExpression::MapApplication {
    ///         function_expression: Expression::Call {
    ///             id: String::from("+"),
    ///             typing: Type::Abstract(
    ///                 vec![Type::Integer, Type::Integer],
    ///                 Box::new(Type::Integer)
    ///             ),
    ///             location: Location::default(),
    ///         },
    ///         inputs: vec![
    ///             StreamExpression::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             StreamExpression::NodeApplication {
    ///                 node: String::from("my_node"),
    ///                 inputs: vec![
    ///                     StreamExpression::SignalCall {
    ///                         id: String::from("s"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     },
    ///                     StreamExpression::MapApplication {
    ///                         function_expression: Expression::Call {
    ///                             id: String::from("*2"),
    ///                             typing: Type::Abstract(
    ///                                 vec![Type::Integer],
    ///                                 Box::new(Type::Integer),
    ///                             ),
    ///                             location: Location::default(),
    ///                         },
    ///                         inputs: vec![StreamExpression::SignalCall {
    ///                             id: String::from("v"),
    ///                             typing: Type::Integer,
    ///                             location: Location::default(),
    ///                         }],
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     },
    ///                 ],
    ///                 signal: String::from("o"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///         ],
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///     },
    ///     location: Location::default(),
    /// };
    /// let unitary_node_1 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: vec![equation_1.clone()],
    ///     location: Location::default(),
    /// };
    /// let equation_2 = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("y"),
    ///     signal_type: Type::Integer,
    ///     expression: StreamExpression::NodeApplication {
    ///         node: String::from("other_node"),
    ///         inputs: vec![
    ///             StreamExpression::MapApplication {
    ///                 function_expression: Expression::Call {
    ///                     id: String::from("-1"),
    ///                     typing: Type::Abstract(
    ///                         vec![Type::Integer],
    ///                         Box::new(Type::Integer),
    ///                     ),
    ///                     location: Location::default(),
    ///                 },
    ///                 inputs: vec![StreamExpression::SignalCall {
    ///                     id: String::from("g"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 }],
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             StreamExpression::SignalCall {
    ///                 id: String::from("v"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///         ],
    ///         signal: String::from("o"),
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///             },
    ///     location: Location::default(),
    /// };
    /// let unitary_node_2 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("y"),
    ///     inputs: vec![(String::from("v"), Type::Integer), (String::from("g"), Type::Integer)],
    ///     scheduled_equations: vec![equation_2.clone()],
    ///     location: Location::default(),
    /// };
    /// let mut node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![
    ///         (String::from("s"), Type::Integer),
    ///         (String::from("v"), Type::Integer),
    ///         (String::from("g"), Type::Integer),
    ///     ],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("x"),
    ///             equation_1.clone()
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             equation_2.clone()
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::from([(String::from("x"), unitary_node_1), (String::from("y"), unitary_node_2)]),
    ///     location: Location::default(),
    /// };
    /// node.normalize(&unitary_nodes_used_inputs);
    ///
    /// let equations_1 = vec![
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_1"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("*2"),
    ///                 typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![StreamExpression::SignalCall {
    ///                 id: String::from("v"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_2"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::NodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("s"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Output,
    ///         id: String::from("x"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("+"),
    ///                 typing: Type::Abstract(
    ///                     vec![Type::Integer, Type::Integer],
    ///                     Box::new(Type::Integer)
    ///                 ),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![
    ///                 StreamExpression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_2"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     }
    /// ];
    /// let unitary_node_1 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: equations_1,
    ///     location: Location::default(),
    /// };
    /// let equations_2 = vec![
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("-1"),
    ///                 typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![StreamExpression::SignalCall {
    ///                 id: String::from("g"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Output,
    ///         id: String::from("y"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::NodeApplication {
    ///             node: String::from("other_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("v"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    /// ];
    /// let unitary_node_2 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("y"),
    ///     inputs: vec![(String::from("v"), Type::Integer), (String::from("g"), Type::Integer)],
    ///     scheduled_equations: equations_2,
    ///     location: Location::default(),
    /// };
    /// let control = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![
    ///         (String::from("s"), Type::Integer),
    ///         (String::from("v"), Type::Integer),
    ///         (String::from("g"), Type::Integer),
    ///     ],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("x"),
    ///             equation_1
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             equation_2
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::from([(String::from("x"), unitary_node_1), (String::from("y"), unitary_node_2)]),
    ///     location: Location::default(),
    /// };
    /// assert_eq!(node, control);
    /// ```
    pub fn normalize(
        &mut self,
        unitary_nodes_used_inputs: &HashMap<String, HashMap<String, Vec<bool>>>,
    ) {
        self.unitary_nodes
            .values_mut()
            .for_each(|unitary_node| unitary_node.normalize(unitary_nodes_used_inputs))
    }
}

#[cfg(test)]
mod add_unitary_node {
    use std::collections::HashMap;

    use crate::common::{
        color::Color, graph::Graph, location::Location, scope::Scope, type_system::Type,
    };
    use crate::ir::{
        equation::Equation, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_add_unitary_node_computing_output() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.add_unitary_node(String::from("o1"), &mut graph, &mut errors)
            .unwrap();

        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            scheduled_equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("i1"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            ],
            location: Location::default(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([(String::from("o1"), unitary_node)]),
            location: Location::default(),
        };

        assert_eq!(node, control)
    }

    #[test]
    fn should_be_scheduled() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.add_unitary_node(String::from("o1"), &mut graph, &mut errors)
            .unwrap();

        let unitary_node = node.unitary_nodes.get(&String::from("o1")).unwrap();
        let schedule = unitary_node
            .scheduled_equations
            .iter()
            .map(|equation| &equation.id)
            .collect::<Vec<_>>();

        let test = graph
            .get_edges()
            .iter()
            .filter_map(|(v1, v2, _)| {
                schedule
                    .iter()
                    .position(|id| id.eq(&v1))
                    .map(|i1| schedule.iter().position(|id| id.eq(&v2)).map(|i2| (i1, i2)))
            })
            .filter_map(|o| o)
            .all(|(i1, i2)| i2 <= i1);

        assert!(test)
    }

    #[test]
    fn should_inform_of_causality_error() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("o1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.add_unitary_node(String::from("o1"), &mut graph, &mut errors)
            .unwrap_err()
    }
}

#[cfg(test)]
mod generate_unitary_nodes {
    use crate::common::{
        color::Color, graph::Graph, location::Location, scope::Scope, type_system::Type,
    };
    use crate::ir::{
        equation::Equation, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_generate_unitary_nodes_as_expected() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.generate_unitary_nodes(&mut graph, &mut errors)
            .unwrap();

        let unitary_node_1 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            scheduled_equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("i1"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            ],
            location: Location::default(),
        };
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o2"),
            inputs: vec![(String::from("i2"), Type::Integer)],
            scheduled_equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o2"),
                signal_type: Type::Integer,
                expression: StreamExpression::SignalCall {
                    id: String::from("i2"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            location: Location::default(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("o2"), unitary_node_2),
                (String::from("o1"), unitary_node_1),
            ]),
            location: Location::default(),
        };

        assert_eq!(node, control)
    }

    #[test]
    fn should_generate_unitary_nodes_for_every_output() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.generate_unitary_nodes(&mut graph, &mut errors)
            .unwrap();

        let mut output_equations = node
            .unscheduled_equations
            .iter()
            .filter(|(_, equation)| equation.scope.eq(&Scope::Output));

        assert!(output_equations.all(|(signal, _)| node.unitary_nodes.contains_key(signal)))
    }

    #[test]
    fn should_raise_error_when_not_causal() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("o1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.generate_unitary_nodes(&mut graph, &mut errors)
            .unwrap_err()
    }

    #[test]
    fn should_raise_error_for_unused_signals() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("i1"), 0);

        node.generate_unitary_nodes(&mut graph, &mut errors)
            .unwrap_err()
    }
}
