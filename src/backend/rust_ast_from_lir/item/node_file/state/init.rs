use crate::common::convert_case::camel_case;
use crate::lir::item::node_file::state::init::{Init, StateElementInit};
use crate::rust_ast::block::Block;
use crate::rust_ast::expression::{Expression, FieldExpression};
use crate::rust_ast::item::implementation::AssociatedItem;
use crate::rust_ast::item::signature::Signature;
use crate::rust_ast::r#type::Type as RustASTType;
use crate::rust_ast::statement::Statement;

/// Transform LIR init into RustAST implementation method.
pub fn rust_ast_from_lir(init: Init) -> AssociatedItem {
    let signature = Signature {
        public_visibility: true,
        name: String::from("init"),
        receiver: None,
        inputs: vec![],
        output: RustASTType::Identifier {
            identifier: camel_case(&format!("{}State", init.node_name)),
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
                        identifier: camel_case(&format!("{}State::init", node_name)),
                    }),
                    arguments: vec![],
                },
            },
        })
        .collect();
    let statement = Statement::ExpressionLast(Expression::Structure {
        name: camel_case(&format!("{}State", init.node_name)),
        fields,
    });
    let body = Block {
        statements: vec![statement],
    };
    AssociatedItem::AssociatedMethod {
        attributes: vec![],
        signature,
        body,
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::node_file::state::init::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::lir::item::node_file::state::init::{Init, StateElementInit};
    use crate::rust_ast::block::Block;
    use crate::rust_ast::expression::{Expression, FieldExpression};
    use crate::rust_ast::item::implementation::AssociatedItem;
    use crate::rust_ast::item::signature::Signature;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::rust_ast::statement::Statement;

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
        let init = Init {
            postconditions: vec![],
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
            attributes: vec![],
            signature: Signature {
                public_visibility: true,
                name: format!("init"),
                receiver: None,
                inputs: vec![],
                output: RustASTType::Identifier {
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
        assert_eq!(rust_ast_from_lir(init), control)
    }
}
