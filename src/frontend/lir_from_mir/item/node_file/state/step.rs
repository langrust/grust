use crate::frontend::lir_from_mir::expression::lir_from_mir as expression_lir_from_mir;
use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::frontend::lir_from_mir::statement::lir_from_mir as statement_lir_from_mir;
use crate::lir::block::Block;
use crate::lir::expression::{Expression, FieldExpression};
use crate::lir::item::implementation::AssociatedItem;
use crate::lir::item::signature::{Receiver, Signature};
use crate::lir::r#type::Type as LIRType;
use crate::lir::statement::Statement;
use crate::mir::item::node_file::state::step::{StateElementStep, Step};

/// Transform MIR step into LIR implementation method.
pub fn lir_from_mir(step: Step) -> AssociatedItem {
    let signature = Signature {
        public_visibility: true,
        name: String::from("step"),
        receiver: Some(Receiver {
            reference: false,
            mutable: false,
        }),
        inputs: vec![(
            String::from("input"),
            LIRType::Identifier {
                identifier: step.node_name.clone() + "Input",
            },
        )],
        output: type_lir_from_mir(step.output_type), // TODO : ADD TUPLE
    };
    let mut statements = step
        .body
        .into_iter()
        .map(|statement| statement_lir_from_mir(statement))
        .collect::<Vec<_>>();

    let fields = step
        .state_elements_step
        .into_iter()
        .map(
            |StateElementStep {
                 identifier,
                 expression,
             }| FieldExpression {
                name: identifier,
                expression: expression_lir_from_mir(expression),
            },
        )
        .collect();
    let statement = Statement::ExpressionLast(Expression::Tuple {
        elements: vec![
            expression_lir_from_mir(step.output_expression),
            Expression::Structure {
                name: step.node_name + "State",
                fields,
            },
        ],
    });

    statements.push(statement);

    let body = Block { statements };
    AssociatedItem::AssociatedMethod { signature, body }
}
