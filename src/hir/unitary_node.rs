use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
};
use crate::hir::{contract::Contract, equation::Equation, memory::Memory, once_cell::OnceCell};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust unitary node HIR.
pub struct UnitaryNode {
    /// The unitary node id in Symbol Table.
    pub unitary_node_id: usize,
    /// Mother node identifier.
    pub node_id: usize,
    /// Output signal identifier.
    pub output_id: usize,
    /// Unitary node's inputs identifiers and their types.
    pub inputs: Vec<(usize, Type)>,
    /// Unitary node's equations.
    pub equations: Vec<Equation>,
    /// Unitary node's memory.
    pub memory: Memory,
    /// Mother node location.
    pub location: Location,
    /// Unitary node dependency graph.
    pub graph: OnceCell<Graph<Color>>,
    /// Unitary node contracts.
    pub contract: Contract,
}

impl UnitaryNode {
    /// Return vector of unitary node's signals.
    pub fn get_signals(&self) -> Vec<usize> {
        let mut signals = vec![];
        self.inputs.iter().for_each(|(signal, _)| {
            signals.push(signal.clone());
        });
        self.equations.iter().for_each(|equation| {
            signals.push(equation.id.clone());
        });
        signals
    }

    /// Tells if two unscheduled unitary nodes are equal.
    pub fn eq_unscheduled(&self, other: &UnitaryNode) -> bool {
        self.node_id == other.node_id
            && self.output_id == other.output_id
            && self.inputs == other.inputs
            && self.equations.len() == other.equations.len()
            && self.equations.iter().all(|equation| {
                other
                    .equations
                    .iter()
                    .any(|other_equation| equation == other_equation)
            })
            && self.memory == other.memory
            && self.location == other.location
    }
}

// #[cfg(test)]
// mod get_signals {

//     use crate::ast::expression::Expression;
//     use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
//     use crate::hir::{
//         dependencies::Dependencies, equation::Equation, memory::Memory, once_cell::OnceCell,
//         signal::Signal, stream_expression::StreamExpression, unitary_node::UnitaryNode,
//     };

//     #[test]
//     fn should_return_all_signals_from_unitary_node() {
//         let equation = Equation {
//             scope: Scope::Output,
//             id: String::from("x"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::FunctionApplication {
//                 function_expression: Expression::Call {
//                     id: String::from("+"),
//                     typing: Some(Type::Abstract(
//                         vec![Type::Integer, Type::Integer],
//                         Box::new(Type::Integer),
//                     )),
//                     location: Location::default(),
//                 },
//                 inputs: vec![
//                     StreamExpression::SignalCall {
//                         signal: Signal {
//                             id: String::from("s"),
//                             scope: Scope::Input,
//                         },
//                         typing: Type::Integer,
//                         location: Location::default(),
//                         dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
//                     },
//                     StreamExpression::FollowedBy {
//                         constant: Constant::Integer(0),
//                         expression: Box::new(StreamExpression::SignalCall {
//                             signal: Signal {
//                                 id: String::from("v"),
//                                 scope: Scope::Input,
//                             },
//                             typing: Type::Integer,
//                             location: Location::default(),
//                             dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
//                         }),
//                         typing: Type::Integer,
//                         location: Location::default(),
//                         dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
//                     },
//                 ],
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![
//                     (String::from("s"), 0),
//                     (String::from("v"), 1),
//                 ]),
//             },
//             location: Location::default(),
//         };
//         let unitary_node = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![
//                 (String::from("s"), Type::Integer),
//                 (String::from("v"), Type::Integer),
//             ],
//             equations: vec![equation],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };
//         let mut signals = unitary_node.get_signals();

//         let mut control = vec![String::from("x"), String::from("s"), String::from("v")];
//         assert_eq!(signals.len(), control.len());
//         while let Some(id) = signals.pop() {
//             let index = control.iter().position(|r| r.eq(&id)).unwrap();
//             let _ = control.remove(index);
//         }
//     }
// }

// #[cfg(test)]
// mod eq_unscheduled {

//     use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
//     use crate::hir::{
//         dependencies::Dependencies, equation::Equation, memory::Memory, once_cell::OnceCell,
//         signal::Signal, stream_expression::StreamExpression, unitary_node::UnitaryNode,
//     };

//     #[test]
//     fn should_return_true_for_strictly_equal_unitary_nodes() {
//         let equation_1 = Equation {
//             scope: Scope::Output,
//             id: String::from("x"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::SignalCall {
//                 signal: Signal {
//                     id: String::from("y"),
//                     scope: Scope::Local,
//                 },
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
//             },
//             location: Location::default(),
//         };
//         let equation_2 = Equation {
//             scope: Scope::Local,
//             id: String::from("y"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::FollowedBy {
//                 constant: Constant::Integer(0),
//                 expression: Box::new(StreamExpression::SignalCall {
//                     signal: Signal {
//                         id: String::from("v"),
//                         scope: Scope::Input,
//                     },
//                     typing: Type::Integer,
//                     location: Location::default(),
//                     dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
//                 }),
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
//             },
//             location: Location::default(),
//         };
//         let unitary_node = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1, equation_2],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         let other = unitary_node.clone();

//         assert!(unitary_node.eq_unscheduled(&other))
//     }

//     #[test]
//     fn should_return_true_for_strictly_equal_unitary_nodes_unscheduled() {
//         let equation_1 = Equation {
//             scope: Scope::Output,
//             id: String::from("x"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::SignalCall {
//                 signal: Signal {
//                     id: String::from("y"),
//                     scope: Scope::Local,
//                 },
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
//             },
//             location: Location::default(),
//         };
//         let equation_2 = Equation {
//             scope: Scope::Local,
//             id: String::from("y"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::FollowedBy {
//                 constant: Constant::Integer(0),
//                 expression: Box::new(StreamExpression::SignalCall {
//                     signal: Signal {
//                         id: String::from("v"),
//                         scope: Scope::Input,
//                     },
//                     typing: Type::Integer,
//                     location: Location::default(),
//                     dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
//                 }),
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
//             },
//             location: Location::default(),
//         };
//         let unitary_node = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1.clone(), equation_2.clone()],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         let other = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_2, equation_1],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         assert!(unitary_node.eq_unscheduled(&other))
//     }

//     #[test]
//     fn should_return_false_for_missing_equations() {
//         let equation_1 = Equation {
//             scope: Scope::Output,
//             id: String::from("x"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::SignalCall {
//                 signal: Signal {
//                     id: String::from("y"),
//                     scope: Scope::Local,
//                 },
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
//             },
//             location: Location::default(),
//         };
//         let equation_2 = Equation {
//             scope: Scope::Local,
//             id: String::from("y"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::FollowedBy {
//                 constant: Constant::Integer(0),
//                 expression: Box::new(StreamExpression::SignalCall {
//                     signal: Signal {
//                         id: String::from("v"),
//                         scope: Scope::Input,
//                     },
//                     typing: Type::Integer,
//                     location: Location::default(),
//                     dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
//                 }),
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
//             },
//             location: Location::default(),
//         };
//         let unitary_node = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1.clone(), equation_2],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         let other = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1.clone(), equation_1],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         assert!(!unitary_node.eq_unscheduled(&other))
//     }

//     #[test]
//     fn should_return_false_for_too_much_equations() {
//         let equation_1 = Equation {
//             scope: Scope::Output,
//             id: String::from("x"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::SignalCall {
//                 signal: Signal {
//                     id: String::from("y"),
//                     scope: Scope::Local,
//                 },
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
//             },
//             location: Location::default(),
//         };
//         let equation_2 = Equation {
//             scope: Scope::Local,
//             id: String::from("y"),
//             signal_type: Type::Integer,
//             expression: StreamExpression::FollowedBy {
//                 constant: Constant::Integer(0),
//                 expression: Box::new(StreamExpression::SignalCall {
//                     signal: Signal {
//                         id: String::from("v"),
//                         scope: Scope::Input,
//                     },
//                     typing: Type::Integer,
//                     location: Location::default(),
//                     dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
//                 }),
//                 typing: Type::Integer,
//                 location: Location::default(),
//                 dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
//             },
//             location: Location::default(),
//         };
//         let unitary_node = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1.clone(), equation_2.clone()],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         let other = UnitaryNode {
//             contract: Default::default(),
//             node_id: String::from("test"),
//             output_id: String::from("x"),
//             inputs: vec![(String::from("v"), Type::Integer)],
//             equations: vec![equation_1.clone(), equation_2.clone(), equation_1],
//             memory: Memory::new(),
//             location: Location::default(),
//             graph: OnceCell::new(),
//         };

//         assert!(!unitary_node.eq_unscheduled(&other))
//     }
// }
