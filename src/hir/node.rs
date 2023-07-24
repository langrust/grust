use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
};
use crate::hir::{equation::Equation, unitary_node::UnitaryNode};

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
