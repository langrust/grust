use crate::{ast::statement::Statement, lir::statement::Statement as LIRStatement};

use super::expression::lir_from_hir as expression_lir_from_hir;

/// Transform HIR statement into LIR statement.
pub fn lir_from_hir(statement: Statement) -> LIRStatement {
    let Statement { id, expression, .. } = statement;
    LIRStatement::Let {
        identifier: id,
        expression: expression_lir_from_hir(expression),
    }
}

#[cfg(test)]
mod lir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, statement::Statement as ASTStatement},
        common::{constant::Constant, location::Location, r#type::Type},
        frontend::lir_from_hir::statement::lir_from_hir,
        lir::{expression::Expression, statement::Statement},
    };

    #[test]
    fn should_transform_ast_statement_of_constant_into_mir_let_statement() {
        let statement = ASTStatement {
            id: format!("y"),
            expression: ASTExpression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            },
            element_type: Type::Integer,
            location: Location::default(),
        };
        let control = Statement::Let {
            identifier: format!("y"),
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        assert_eq!(lir_from_hir(statement), control)
    }
}
