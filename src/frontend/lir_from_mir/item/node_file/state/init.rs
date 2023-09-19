use crate::common::r#type::Type;
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
        output: LIRType::Owned(Type::Structure(init.node_name.clone() + "State")),
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
