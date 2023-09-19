use crate::frontend::lir_from_mir::{
    block::lir_from_mir as block_lir_from_mir, r#type::lir_from_mir as type_lir_from_mir,
};
use crate::lir::item::{function::Function as LIRFunction, signature::Signature};
use crate::mir::item::function::Function;

/// Transform MIR function into LIR function.
pub fn lir_from_mir(function: Function) -> LIRFunction {
    let inputs = function
        .inputs
        .into_iter()
        .map(|(name, r#type)| (name, type_lir_from_mir(r#type)))
        .collect();
    let signature = Signature {
        public_visibility: true,
        name: function.name,
        receiver: None,
        inputs,
        output: type_lir_from_mir(function.output),
    };
    LIRFunction {
        signature,
        body: block_lir_from_mir(function.body),
    }
}
