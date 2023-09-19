use crate::lir::block::Block as LIRBlock;
use crate::mir::block::Block;

use super::statement::lir_from_mir as statement_lir_from_mir;

/// Transform MIR block into LIR block.
pub fn lir_from_mir(block: Block) -> LIRBlock {
    let statements = block
        .statements
        .into_iter()
        .map(|statement| statement_lir_from_mir(statement))
        .collect();
    LIRBlock { statements }
}
