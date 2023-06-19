use std::collections::HashMap;

use crate::common::{
    color::Color, graph::Graph, location::Location, user_defined_type::UserDefinedType,
};
use crate::error::Error;
use crate::ir::{function::Function, node::Node};

#[derive(Debug, PartialEq)]
/// A LanGRust [File] is composed of functions nodes,
/// types defined by the user and an optional component.
pub struct File {
    /// Program types.
    pub user_defined_types: Vec<UserDefinedType>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program nodes. They are functional requirements.
    pub nodes: Vec<Node>,
    /// Program component. It represents the system.
    pub component: Option<Node>,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Generate dependencies graph for every nodes/component.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, function::Function,
    ///     file::File, node::Node, statement::Statement, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     color::Color, graph::Graph, location::Location, scope::Scope, type_system::Type
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
    ///
    /// let function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         (
    ///             String::from("x"),
    ///             Statement {
    ///                 id: String::from("x"),
    ///                 element_type: Type::Integer,
    ///                 expression: Expression::Call {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         }
    ///     ),
    ///     location: Location::default(),
    /// };
    ///
    /// let mut file = File {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    ///
    /// let nodes_graphs = file.generate_dependencies_graphs(&mut errors).unwrap();
    ///
    /// let graph = nodes_graphs.get(&String::from("test")).unwrap();
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
    pub fn generate_dependencies_graphs(
        &self,
        errors: &mut Vec<Error>,
    ) -> Result<HashMap<String, Graph<Color>>, ()> {
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

        // every nodes complete their dependencies graphs
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

        // optional component completes its dependencies graph
        component.as_ref().map_or(Ok(()), |component| {
            component.add_all_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        // return direct dependencies graphs
        Ok(nodes_graphs)
    }

    /// Normalize IR file.
    ///
    /// Normalize all nodes of a file as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// function test(i: int) -> int {
    ///     let x: int = i;
    ///     return x;
    /// }
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
    ///     equation::Equation, expression::Expression, file::File, function::Function,
    ///     node::Node, statement::Statement, stream_expression::StreamExpression,
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
    /// let node = Node {
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
    /// let function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         (
    ///             String::from("x"),
    ///             Statement {
    ///                 id: String::from("x"),
    ///                 element_type: Type::Integer,
    ///                 expression: Expression::Call {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         }
    ///     ),
    ///     location: Location::default(),
    /// };
    /// let mut file = File {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    /// file.normalize();
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
    /// let node = Node {
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
    /// let function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         (
    ///             String::from("x"),
    ///             Statement {
    ///                 id: String::from("x"),
    ///                 element_type: Type::Integer,
    ///                 expression: Expression::Call {
    ///                     id: String::from("i"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         }
    ///     ),
    ///     location: Location::default(),
    /// };
    /// let control = File {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    /// assert_eq!(file, control);
    /// ```
    pub fn normalize(&mut self) {
        self.nodes.iter_mut().for_each(|node| node.normalize())
    }
}
