use crate::{ast::statement::Statement, lir::statement::Statement as MIRStatement};

use super::expression::mir_from_hir as expression_mir_from_hir;

/// Transform HIR statement into MIR statement.
pub fn mir_from_hir(statement: Statement) -> MIRStatement {
    let Statement { id, expression, .. } = statement;
    MIRStatement::Let {
        identifier: id,
        expression: expression_mir_from_hir(expression),
    }
}

#[cfg(test)]
mod mir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, statement::Statement as ASTStatement},
        common::{constant::Constant, location::Location, r#type::Type},
        frontend::mir_from_hir::statement::mir_from_hir,
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
        assert_eq!(mir_from_hir(statement), control)
    }
}
