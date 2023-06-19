use std::collections::HashMap;

use crate::common::{location::Location, scope::Scope, type_system::Type};
use crate::ir::{identifier_creator::IdentifierCreator, stream_expression::StreamExpression};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust equation AST.
pub struct Equation {
    /// Signal's scope.
    pub scope: Scope,
    /// Identifier of the signal.
    pub id: String,
    /// Signal type.
    pub signal_type: Type,
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    /// Equation location.
    pub location: Location,
}

impl Equation {
    /// Normalize IR equations.
    ///
    /// Normalize IR equations as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::{HashSet, HashMap};
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, identifier_creator::IdentifierCreator,
    ///     stream_expression::StreamExpression,
    /// };
    ///
    /// let mut identifier_creator = IdentifierCreator {
    ///     signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
    /// };
    /// let unitary_nodes_used_inputs = HashMap::from([(
    ///     String::from("my_node"),
    ///     HashMap::from([(String::from("o"), vec![true, true])]),
    /// )]);
    ///
    /// let equation = Equation {
    ///     scope: Scope::Local,
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
    /// let equations = equation.normalize(&mut identifier_creator, &unitary_nodes_used_inputs);
    ///
    /// let control = vec![
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
    ///         expression: StreamExpression::UnitaryNodeApplication {
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
    ///         scope: Scope::Local,
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
    /// assert_eq!(equations, control);
    /// ```
    pub fn normalize(
        self,
        identifier_creator: &mut IdentifierCreator,
        unitary_nodes_used_inputs: &HashMap<String, HashMap<String, Vec<bool>>>,
    ) -> Vec<Equation> {
        let Equation {
            scope,
            id,
            signal_type,
            mut expression,
            location,
        } = self;

        // normalize expression and get additional equations
        let mut equations = expression.normalize(identifier_creator, unitary_nodes_used_inputs);

        // recreate the new equation with modified expression
        let normalized_equation = Equation {
            scope,
            id,
            signal_type,
            expression,
            location,
        };

        // push normalized equation in the equations storage (in scheduling order)
        equations.push(normalized_equation);

        // return equations
        equations
    }
}
