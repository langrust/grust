use crate::common::{location::Location, type_system::Type};
use crate::ir::equation::Equation;

use super::identifier_creator::IdentifierCreator;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust unitary node IR.
pub struct UnitaryNode {
    /// Mother node identifier.
    pub node_id: String,
    /// Output signal identifier.
    pub output_id: String,
    /// Unitary node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Unitary node's scheduled equations.
    pub scheduled_equations: Vec<Equation>,
    /// Mother node location.
    pub location: Location,
}

impl UnitaryNode {
    /// Normalize IR unitary nodes.
    ///
    /// Normalize IR unitary node's equations as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::HashSet;
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
    ///
    /// let equation = Equation {
    ///     scope: Scope::Local,
    ///     id: String::from("x"),
    ///     signal_type: Type::Integer,
    ///     expression: StreamExpression::MapApplication {
    ///         function_expression: Expression::Call {
    ///             id: String::from("+"),
    ///             typing: Type::Abstract(
    ///                 Box::new(Type::Integer),
    ///                 Box::new(Type::Abstract(
    ///                     Box::new(Type::Integer),
    ///                     Box::new(Type::Integer),
    ///                 )),
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
    ///                         id: String::from("x"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     },
    ///                     StreamExpression::MapApplication {
    ///                         function_expression: Expression::Call {
    ///                             id: String::from("*2"),
    ///                             typing: Type::Abstract(
    ///                                 Box::new(Type::Integer),
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
    /// let mut unitary_node = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: vec![equation],
    ///     location: Location::default(),
    /// };
    /// unitary_node.normalize();
    ///
    /// let equations = vec![
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_1"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("*2"),
    ///                 typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
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
    ///                     id: String::from("x"),
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
    ///         scope: Scope::Local,
    ///         id: String::from("x"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("+"),
    ///                 typing: Type::Abstract(
    ///                     Box::new(Type::Integer),
    ///                     Box::new(Type::Abstract(
    ///                         Box::new(Type::Integer),
    ///                         Box::new(Type::Integer),
    ///                     )),
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
    /// let control = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: equations,
    ///     location: Location::default(),
    /// };
    /// assert_eq!(unitary_node, control);
    /// ```
    pub fn normalize(&mut self) {
        let mut identifier_creator = IdentifierCreator::new(self);

        let UnitaryNode {
            scheduled_equations,
            ..
        } = self;

        *scheduled_equations = scheduled_equations
            .clone()
            .into_iter()
            .flat_map(|equation| equation.normalize(&mut identifier_creator))
            .collect();
    }
}
