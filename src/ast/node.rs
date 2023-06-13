use std::collections::HashMap;

use crate::ast::{
    equation::Equation, location::Location, node_description::NodeDescription, scope::Scope,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::color::Color;
use crate::common::context::Context;
use crate::common::graph::neighbor::Neighbor;
use crate::common::graph::Graph;
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's equations.
    pub equations: Vec<(String, Equation)>,
    /// Node location.
    pub location: Location,
}

impl Node {
    /// [Type] the node.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, equation::Equation, location::Location, node::Node,
    ///     node_description::NodeDescription, scope::Scope,
    ///     stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     NodeDescription {
    ///         is_component: false,
    ///         inputs: vec![(String::from("i"), Type::Integer)],
    ///         outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///         locals: HashMap::from([(String::from("x"), Type::Integer)]),
    ///     }
    /// );
    /// let global_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
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
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// node.typing(&nodes_context, &global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            id,
            equations,
            location,
            ..
        } = self;

        // get the description of the node
        let NodeDescription {
            inputs,
            outputs,
            locals,
            ..
        } = nodes_context.get_node_or_error(id, location.clone(), errors)?;

        // create signals context: inputs + outputs + locals
        let mut signals_context = HashMap::new();
        inputs
            .iter()
            .map(|(name, input_type)| {
                signals_context.insert_unique(
                    name.clone(),
                    input_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;
        signals_context.combine_unique(outputs.clone(), location.clone(), errors)?;
        signals_context.combine_unique(locals.clone(), location.clone(), errors)?;

        // type all equations
        equations
            .iter_mut()
            .map(|(_, equation)| {
                equation.typing(
                    nodes_context,
                    &signals_context,
                    global_context,
                    user_types_context,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create a [NodeDescription] from a [Node]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, node::Node, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
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
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let control = NodeDescription {
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///     locals: HashMap::from([(String::from("x"), Type::Integer)]),
    /// };
    ///
    /// let node_description = node.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Node {
            is_component,
            inputs,
            equations,
            location,
            ..
        } = self;

        // differenciate output form local signals
        let mut outputs = HashMap::new();
        let mut locals = HashMap::new();

        // create signals context: inputs + outputs + locals
        // and check that no signal is duplicated
        let mut signals_context = HashMap::new();

        // add inputs in signals context
        inputs
            .iter()
            .map(|(id, signal_type)| {
                signals_context.insert_unique(
                    id.clone(),
                    signal_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // add signals defined by equations in contexts
        equations
            .iter()
            .map(
                |(
                    _,
                    Equation {
                        scope,
                        id,
                        signal_type,
                        location,
                        ..
                    },
                )| {
                    // differenciate output form local signals
                    match scope {
                        Scope::Output => outputs.insert(id.clone(), signal_type.clone()),
                        Scope::Local => locals.insert(id.clone(), signal_type.clone()),
                        _ => unreachable!(),
                    };
                    // check that no signal is duplicated
                    signals_context.insert_unique(
                        id.clone(),
                        signal_type.clone(),
                        location.clone(),
                        errors,
                    )
                },
            )
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        Ok(NodeDescription {
            is_component: is_component.clone(),
            inputs: inputs.clone(),
            outputs,
            locals,
        })
    }

    /// Determine all undefined types in node
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, node::Node,
    ///     equation::Equation, stream_expression::StreamExpression, scope::Scope,
    ///     location::Location, type_system::Type, user_defined_type::UserDefinedType,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// user_types_context.insert(
    ///     String::from("Point"),
    ///     UserDefinedType::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     }
    /// );
    ///
    /// let mut node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::NotDefinedYet(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Structure(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// node
    ///     .resolve_undefined_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(node, control);
    /// ```
    pub fn resolve_undefined_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            inputs,
            equations,
            location,
            ..
        } = self;

        // determine inputs types
        inputs
            .iter_mut()
            .map(|(_, input_type)| {
                input_type.resolve_undefined(location.clone(), user_types_context, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine equations types
        equations
            .iter_mut()
            .map(|(_, equation)| equation.resolve_undefined_types(user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     constant::Constant, node::Node, equation::Equation, location::Location,
    ///     node_description::NodeDescription, scope::Scope,
    ///     stream_expression::StreamExpression, type_system::Type,
    /// };
    /// use grustine::common::{color::Color, graph::Graph};
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
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
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let graph = node.create_initialized_graph(&mut errors).unwrap();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::White);
    /// control.add_vertex(String::from("x"), Color::White);
    /// control.add_vertex(String::from("i"), Color::White);
    ///
    /// assert_eq!(graph, control);
    /// ```
    pub fn create_initialized_graph(&self, errors: &mut Vec<Error>) -> Result<Graph<Color>, ()> {
        // create an empty graph
        let mut graph = Graph::new();

        // get node's signals
        let NodeDescription {
            inputs,
            outputs,
            locals,
            ..
        } = self.into_node_description(errors)?;

        // add input signals as vertices
        for (input, _) in inputs {
            graph.add_vertex(input.clone(), Color::White);
        }

        // add output signals as vertices
        for (output, _) in outputs {
            graph.add_vertex(output.clone(), Color::White);
        }

        // add local signals as vertices
        for (local, _) in locals {
            graph.add_vertex(local.clone(), Color::White);
        }

        // return graph
        Ok(graph)
    }

    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     constant::Constant, node::Node, equation::Equation, location::Location,
    ///     node_description::NodeDescription, scope::Scope,
    ///     stream_expression::StreamExpression, type_system::Type,
    /// };
    /// use grustine::common::{color::Color, graph::Graph};
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
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
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     node
    /// );
    /// let node = nodes_context.get(&String::from("test")).unwrap();
    ///
    /// let graph = node.create_initialized_graph(&mut errors).unwrap();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph(&mut errors).unwrap();
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
            equations,
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

                equations.iter().find(|(id, _)| id.eq(signal)).map_or(
                    Ok(()),
                    |(_, equation)| {
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
                    },
                )?;

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

    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     constant::Constant, node::Node, equation::Equation, location::Location,
    ///     node_description::NodeDescription, scope::Scope,
    ///     stream_expression::StreamExpression, type_system::Type,
    /// };
    /// use grustine::common::{color::Color, graph::Graph};
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
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
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     node
    /// );
    /// let node = nodes_context.get(&String::from("test")).unwrap();
    ///
    /// let graph = node.create_initialized_graph(&mut errors).unwrap();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph(&mut errors).unwrap();
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
                for Neighbor { id, weight } in vertex.get_neighbors() {
                    // tells if the neighbor is an input
                    let is_input = inputs.iter().any(|(input, _)| *input == id);

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        reduced_graph.add_edge(signal, id, weight);
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
                            |Neighbor { id, weight }| reduced_graph.add_edge(signal, id, weight),
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
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, node::Node,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };

    #[test]
    fn should_type_well_defined_node() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("test"),
            NodeDescription {
                is_component: false,
                inputs: vec![(String::from("i"), Type::Integer)],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::from([(String::from("x"), Type::Integer)]),
            },
        );
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
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
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Some(Type::Integer),
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
                            id: String::from("i"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        node.typing(
            &nodes_context,
            &global_context,
            &user_types_context,
            &mut errors,
        )
        .unwrap();

        assert_eq!(node, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_node() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("test"),
            NodeDescription {
                is_component: false,
                inputs: vec![(String::from("i"), Type::Integer)],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::from([(String::from("x"), Type::Integer)]),
            },
        );
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::Constant {
                            constant: Constant::Float(0.1),
                            typing: None,
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
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        node.typing(
            &nodes_context,
            &global_context,
            &user_types_context,
            &mut errors,
        )
        .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, location::Location, node::Node, node_description::NodeDescription,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
    };

    #[test]
    fn should_return_a_node_description_from_a_node_with_no_duplicates() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
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
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let control = NodeDescription {
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            outputs: HashMap::from([(String::from("o"), Type::Integer)]),
            locals: HashMap::from([(String::from("x"), Type::Integer)]),
        };

        let node_description = node.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }

    #[test]
    fn should_return_a_node_description_from_a_component_with_no_duplicates() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("test"),
            is_component: true,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
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
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let control = NodeDescription {
            is_component: true,
            inputs: vec![(String::from("i"), Type::Integer)],
            outputs: HashMap::from([(String::from("o"), Type::Integer)]),
            locals: HashMap::from([(String::from("x"), Type::Integer)]),
        };

        let node_description = node.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, node::Node, scope::Scope,
        stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
    };
    use std::collections::HashMap;

    #[test]
    fn should_determine_undefined_types_when_in_context() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            UserDefinedType::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::NotDefinedYet(String::from("Point")),
                    expression: StreamExpression::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                        ],
                        typing: None,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            location: Location::default(),
        };

        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Structure(String::from("Point")),
                    expression: StreamExpression::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                        ],
                        typing: None,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            location: Location::default(),
        };

        node.resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(node, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::NotDefinedYet(String::from("Point")),
                    expression: StreamExpression::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ),
                        ],
                        typing: None,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            location: Location::default(),
        };

        node.resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
