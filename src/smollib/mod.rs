mod int;
mod len;
mod list;
mod range;
use std::usize;

use crate::asm::codegen::CodeGen;
use crate::asm::llvm::{user_function_prefix, user_function_prefix_format};
use crate::asm::LLVMCodegenError;
use crate::common::symbol_table::{Symbol, SymbolTableElement, SymbolTableRef};
use crate::common::types::{IdToken, Token};
use crate::lexer::TokenTable;
use crate::typing::{Function, Type};
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

/// Get an smollib function registered in the CodeGen.
/// The smollib functions MUST be initialized before using this macro
macro_rules! get_smollib_func {
    ($cg: expr, $name: expr) => {
        $cg.module.get_function($name.to_string().as_str()).unwrap()
    };
}
pub(super) use get_smollib_func;

trait SmollibFunction {
    fn name(&self) -> &str;
    fn func_type(&self) -> Function;
    fn llvm_type<'ctx>(&self, cg: &CodeGen<'ctx>) -> FunctionType<'ctx>;
    fn build_llvm<'ctx>(
        &self,
        function: FunctionValue<'ctx>,
        cg: &mut CodeGen<'ctx>,
    ) -> Result<(), LLVMCodegenError>;
}

fn get_all_smollib_functions() -> Vec<Box<dyn SmollibFunction>> {
    vec![
        Box::new(len::SmolLen {}),
        Box::new(list::SmolList {}),
        Box::new(range::SmolRange {}),
        Box::new(int::SmolInt {}),
    ]
}

#[allow(unused)]
pub enum SmollibFunctionNames {
    SmolLen,
    SmolList,
    SmolRange,
    SmolInt,
}

impl ToString for SmollibFunctionNames {
    fn to_string(&self) -> String {
        let s = match self {
            SmollibFunctionNames::SmolLen => user_function_prefix_format!(len::SmolLen {}.name()),
            SmollibFunctionNames::SmolList => {
                user_function_prefix_format!(list::SmolList {}.name())
            }
            SmollibFunctionNames::SmolRange => {
                user_function_prefix_format!(range::SmolRange {}.name())
            }
            SmollibFunctionNames::SmolInt => {
                user_function_prefix_format!(int::SmolInt {}.name())
            }
        };
        String::from(s)
    }
}

/// Register smollib names in the token table
pub fn register_smollib_names(token_table: &mut TokenTable) {
    let mut id = usize::MAX;

    for func in get_all_smollib_functions().iter() {
        let func_token = IdToken {
            id,
            name: String::from(func.name()),
        };
        token_table.reserve_smollib_name(func.name(), Token::Identifier(func_token));
        id = id - 1;
    }
}

/// Register smollib functions in the symbol table
pub fn register_smollib_funcs(symbol_table: &mut SymbolTableRef) {
    let mut id = usize::MAX;

    for func in get_all_smollib_functions().iter() {
        // Note: no need to get old type, because first time we define the function, return update in sub node

        let function_type = func.func_type();

        let symbol_table_element = SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from(func.name()),
            symbol_type: Type::Function(Box::from(function_type)),
        };
        symbol_table
            .borrow_mut()
            .insert_symbol(id, symbol_table_element);

        id = id - 1;
    }
}

pub fn register_smollib_funcs_in_module<'ctx>(cg: &mut CodeGen<'ctx>) {
    let funcs = get_all_smollib_functions();

    for func in funcs {
        let func_type = func.llvm_type(cg);
        cg.module.add_function(
            user_function_prefix_format!(func.name()).as_str(),
            func_type,
            None,
        );
    }
}

pub fn build_smollib_llvm<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let funcs = get_all_smollib_functions();

    for func in funcs {
        let func_val = cg
            .module
            .get_function(user_function_prefix_format!(func.name()).as_str())
            .unwrap();
        func.build_llvm(func_val, cg)?;
    }
    return Ok(());
}
