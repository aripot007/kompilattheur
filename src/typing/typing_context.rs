use crate::common::{
    diagnostic::Diagnostic,
    localizable::Localizable,
    symbol_table::{get_symbol, Symbol, SymbolTableElement, SymbolTableRef},
    types::IdToken,
};

use super::{Type, Weak};

pub struct TypingContext {
    pub symbol_table: SymbolTableRef,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
    pub func_id: Option<IdToken>,
}

impl TypingContext {
    /// Get the type associated to a symbol, or create a Variable symbol entry for it with a weak type.
    pub fn get_type_or_create(&mut self, identifier: &IdToken) -> Type {
        if let (_, Some(elt)) = get_symbol(self.symbol_table.clone(), &identifier.id) {
            return elt.symbol_type;
        }

        let t = Type::Weak(Weak::new());

        let symbol_entry = SymbolTableElement {
            symbol: Symbol::Variable(),
            name: identifier.name.clone(),
            symbol_type: t.clone(),
        };

        self.symbol_table
            .borrow_mut()
            .insert_symbol(identifier.id, symbol_entry);

        return t;
    }

    /// Get a symbol type from the symbol table, or create an error
    pub fn get_symbol_type(&mut self, identifier: &IdToken) -> Option<Type> {
        match get_symbol(self.symbol_table.clone(), &identifier.id) {
            (_, Some(elt)) => Some(elt.symbol_type),
            (_, None) => None,
        }
    }

    /// TODO: replace with function that merges types
    pub fn add_symbol(&mut self, identifier: &IdToken, symbol: Symbol, symbol_type: Type) {
        let symbol_entry = SymbolTableElement {
            symbol,
            name: identifier.name.clone(),
            symbol_type,
        };

        self.symbol_table
            .borrow_mut()
            .insert_symbol(identifier.id, symbol_entry);
    }

    pub fn update_function_return(&mut self, identifier: &IdToken, symbol_type: Type) {
        let symbol_table = self.symbol_table.clone();
        let (_, old_symbol_entry) = get_symbol(symbol_table, &identifier.id);
        if let Some(old_symbol_entry) = old_symbol_entry {
            if old_symbol_entry.symbol == Symbol::Function() {
                let symbol_entry = SymbolTableElement {
                    symbol: Symbol::Function(),
                    name: identifier.name.clone(),
                    symbol_type,
                };

                self.symbol_table
                    .borrow_mut()
                    .insert_symbol(identifier.id, symbol_entry);

                return;
            }
            panic!("Symbol is not a function");
        }
        panic!("Function not found");
    }
}
