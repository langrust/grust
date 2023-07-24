use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
    scope::Scope,
};
use crate::error::Error;
use crate::hir::{equation::Equation, memory::Memory, unitary_node::UnitaryNode};

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
    /// Node dependency graph.
    pub graph: OnceCell<Graph<Color>>,
}

impl Node {
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
    /// use once_cell::sync::OnceCell;
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{
    ///     graph::{color::Color, Graph}, location::Location, scope::Scope, r#type::Type,
    /// };
    /// use grustine::hir::{
    ///     dependencies::Dependencies, equation::Equation, node::Node, memory::Memory,
    ///     stream_expression::StreamExpression, unitary_node::UnitaryNode,
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
    ///                     dependencies: Dependencies::from(vec![(String::from("x"), 0)])
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
    ///                     dependencies: Dependencies::from(vec![(String::from("i2"), 0)])
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
    ///                     dependencies: Dependencies::from(vec![(String::from("i1"), 0)])
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    ///     graph: OnceCell::new(),
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
    /// node.graph.set(graph);
    ///
    /// node.generate_unitary_nodes(&mut errors).unwrap();
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
    ///                 dependencies: Dependencies::from(vec![(String::from("i1"), 0)])
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
    ///                 dependencies: Dependencies::from(vec![(String::from("x"), 0)])
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     memory: Memory::new(),
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
    ///                 dependencies: Dependencies::from(vec![(String::from("i2"), 0)])
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     memory: Memory::new(),
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
    ///                     dependencies: Dependencies::from(vec![(String::from("x"), 0)])
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
    ///                     dependencies: Dependencies::from(vec![(String::from("i2"), 0)])
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
    ///                     dependencies: Dependencies::from(vec![(String::from("i1"), 0)])
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::from([(String::from("o2"), unitary_node_2), (String::from("o1"), unitary_node_1)]),
    ///     location: Location::default(),
    ///     graph: OnceCell::new(),
    /// };
    /// let mut graph = Graph::new();
    /// graph.add_vertex(String::from("i1"), Color::Black);
    /// graph.add_vertex(String::from("i2"), Color::Black);
    /// graph.add_vertex(String::from("x"), Color::Black);
    /// graph.add_vertex(String::from("o1"), Color::Black);
    /// graph.add_vertex(String::from("o2"), Color::Black);
    /// graph.add_edge(&String::from("x"), String::from("i1"), 0);
    /// graph.add_edge(&String::from("o1"), String::from("x"), 0);
    /// graph.add_edge(&String::from("o2"), String::from("i2"), 0);
    /// control.graph.set(graph);
    ///
    /// assert_eq!(node, control)
    /// ```
    pub fn generate_unitary_nodes(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
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
            .map(|output| self.add_unitary_node(output, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, ()>>()?;

        // check that every signals are used
        let unused_signals = self.graph.get().unwrap().forgotten_vertices(subgraphs);
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
        let mut subgraph = self.graph.get().unwrap().subgraph_from_vertex(&output);

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
            memory: Memory::new(),
            location: location.clone(),
        };

        // insert it in node's storage
        unitary_nodes.insert(output.clone(), unitary_node);

        Ok(subgraph)
    }

    /// Normalize HIR node.
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
    /// use once_cell::sync::OnceCell;
    /// use std::collections::{HashSet, HashMap};
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, r#type::Type};
    /// use grustine::hir::{
    ///     dependencies::Dependencies, equation::Equation, expression::Expression,
    ///     memory::Memory, node::Node, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
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
    ///                 dependencies: Dependencies::from(vec![])
    ///             },
    ///             StreamExpression::UnitaryNodeApplication {
    ///                 node: String::from("my_node"),
    ///                 inputs: vec![
    ///                     StreamExpression::SignalCall {
    ///                         id: String::from("s"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                         dependencies: Dependencies::from(vec![(String::from("s"), 0)])
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
    ///                             dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///                         }],
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                         dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///                     },
    ///                 ],
    ///                 signal: String::from("o"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)])
    ///             },
    ///         ],
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///         dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)])
    ///     },
    ///     location: Location::default(),
    /// };
    /// let unitary_node_1 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: vec![equation_1.clone()],
    ///     memory: Memory::new(),
    ///     location: Location::default(),
    /// };
    /// let equation_2 = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("y"),
    ///     signal_type: Type::Integer,
    ///     expression: StreamExpression::UnitaryNodeApplication {
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
    ///                     dependencies: Dependencies::from(vec![(String::from("g"), 0)])
    ///                 }],
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::from(vec![(String::from("g"), 0)])
    ///             },
    ///             StreamExpression::SignalCall {
    ///                 id: String::from("v"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///             },
    ///         ],
    ///         signal: String::from("o"),
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///         dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)])
    ///     },
    ///     location: Location::default(),
    /// };
    /// let unitary_node_2 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("y"),
    ///     inputs: vec![(String::from("v"), Type::Integer), (String::from("g"), Type::Integer)],
    ///     scheduled_equations: vec![equation_2.clone()],
    ///     memory: Memory::new(),
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
    ///     graph: OnceCell::new(),
    /// };
    /// node.normalize();
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
    ///                 dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_2"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::UnitaryNodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("s"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("s"), 0)])
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("x_1"), 0)])
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)])
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
    ///                     dependencies: Dependencies::from(vec![])
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_2"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("x_2"), 0)])
    ///                 },
    ///             ],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)])
    ///         },
    ///         location: Location::default(),
    ///     }
    /// ];
    /// let unitary_node_1 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: equations_1,
    ///     memory: Memory::new(),
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
    ///                 dependencies: Dependencies::from(vec![(String::from("g"), 0)])
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("g"), 0)])
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Output,
    ///         id: String::from("y"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::UnitaryNodeApplication {
    ///             node: String::from("other_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("x"), 0)])
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("v"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("v"), 0)])
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)])
    ///         },
    ///         location: Location::default(),
    ///     },
    /// ];
    /// let unitary_node_2 = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("y"),
    ///     inputs: vec![(String::from("v"), Type::Integer), (String::from("g"), Type::Integer)],
    ///     scheduled_equations: equations_2,
    ///     memory: Memory::new(),
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
    ///     graph: OnceCell::new(),
    /// };
    /// assert_eq!(node, control);
    /// ```
    pub fn normalize(&mut self) {
        self.unitary_nodes
            .values_mut()
            .for_each(|unitary_node| unitary_node.normalize())
    }
}

#[cfg(test)]
mod add_unitary_node {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use crate::{
        common::{
            graph::{color::Color, Graph},
            location::Location,
            r#type::Type,
            scope::Scope,
        },
        hir::dependencies::Dependencies,
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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
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
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([(String::from("o1"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
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
        control.graph.set(graph.clone()).unwrap();

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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph.clone()).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
            .unwrap_err()
    }
}

#[cfg(test)]
mod generate_unitary_nodes {
    use once_cell::sync::OnceCell;

    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use crate::{
        common::{
            graph::{color::Color, Graph},
            location::Location,
            r#type::Type,
            scope::Scope,
        },
        hir::dependencies::Dependencies,
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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap();

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
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
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
                    dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
            graph: OnceCell::new(),
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

        control.graph.set(graph.clone()).unwrap();

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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap();

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
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap_err()
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("i1"), 0);

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap_err()
    }
}
