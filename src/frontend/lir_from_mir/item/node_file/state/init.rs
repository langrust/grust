use crate::lir::block::Block;
use crate::lir::expression::{Expression, FieldExpression};
use crate::lir::item::implementation::AssociatedItem;
use crate::lir::item::signature::Signature;
use crate::lir::r#type::Type as LIRType;
use crate::lir::statement::Statement;
use crate::mir::item::node_file::state::init::{Init, StateElementInit};

/// Transform MIR init into LIR implementation method.
pub fn lir_from_mir(init: Init) -> AssociatedItem {
    let signature = Signature {
        public_visibility: true,
        name: String::from("init"),
        receiver: None,
        inputs: vec![],
        output: LIRType::Identifier {
            identifier: init.node_name.clone() + "State",
        },
    };
    let fields = init
        .state_elements_init
        .into_iter()
        .map(|element| match element {
            StateElementInit::BufferInit {
                identifier,
                initial_value,
            } => FieldExpression {
                name: identifier,
                expression: Expression::Literal {
                    literal: initial_value,
                },
            },
            StateElementInit::CalledNodeInit {
                identifier,
                node_name,
            } => FieldExpression {
                name: identifier,
                expression: Expression::FunctionCall {
                    function: Box::new(Expression::Identifier {
                        identifier: node_name + "State::init()",
                    }),
                    arguments: vec![],
                },
            },
        })
        .collect();
    let statement = Statement::ExpressionLast(Expression::Structure {
        name: init.node_name + "State",
        fields,
    });
    let body = Block {
        statements: vec![statement],
    };
    AssociatedItem::AssociatedMethod { signature, body }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::constant::Constant;
    use crate::frontend::lir_from_mir::item::node_file::state::init::lir_from_mir;
    use crate::lir::block::Block;
    use crate::lir::expression::{Expression, FieldExpression};
    use crate::lir::item::implementation::AssociatedItem;
    use crate::lir::item::signature::Signature;
    use crate::lir::r#type::Type as LIRType;
    use crate::lir::statement::Statement;
    use crate::mir::item::node_file::state::init::{Init, StateElementInit};

    #[test]
    fn should_create_lir_associated_method_from_mir_node_init() {
        let init = Init {
            node_name: format!("Node"),
            state_elements_init: vec![
                StateElementInit::BufferInit {
                    identifier: format!("mem_i"),
                    initial_value: Constant::Integer(0),
                },
                StateElementInit::CalledNodeInit {
                    identifier: format!("called_node_state"),
                    node_name: format!("CalledNode"),
                },
            ],
        };
        let control = AssociatedItem::AssociatedMethod {
            signature: Signature {
                public_visibility: true,
                name: format!("init"),
                receiver: None,
                inputs: vec![],
                output: LIRType::Identifier {
                    identifier: format!("NodeState"),
                },
            },
            body: Block {
                statements: vec![Statement::ExpressionLast(Expression::Structure {
                    name: format!("NodeState"),
                    fields: vec![
                        FieldExpression {
                            name: format!("mem_i"),
                            expression: Expression::Literal {
                                literal: Constant::Integer(0),
                            },
                        },
                        FieldExpression {
                            name: format!("called_node_state"),
                            expression: Expression::FunctionCall {
                                function: Box::new(Expression::Identifier {
                                    identifier: format!("CalledNodeState::init"),
                                }),
                                arguments: vec![],
                            },
                        },
                    ],
                })],
            },
        };
        assert_eq!(lir_from_mir(init), control)
    }
}
