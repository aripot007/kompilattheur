use super::{AstNode, Block, Param};
use crate::ast::nodes::{Expression, ExpressionKind};
use crate::typing::Type;
use crate::{
    analysis_table::NonTerminal,
    ast::nodes::{parse_list_filter, Factor, FactorKind, Statement},
    common::{
        localizable::Localizable,
        symbol_table::get_symbol,
        types::{
            file_element::{empty_file_elt, file_element_from},
            FileElement, IdToken, Node, Token, Tree,
        },
    },
    parser::Lexem,
};

pub struct Def {
    pub identifier: FileElement<IdToken>,
    pub params: Vec<Param>,
    pub block: Block,
    pub localization: FileElement<bool>,
}

impl AstNode for Def {
    fn get_string_repr(&self) -> String {
        String::from("Def")
    }
}

impl From<Tree<FileElement<Lexem>>> for Def {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let id_elem = root.borrow().get_children()[1].borrow().get_value();
        let id_token = match id_elem.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!(
                "Unexpected child #1 of <def> node : expected IdToken, got {}",
                t
            ),
        };
        let identifier: FileElement<IdToken> = file_element_from!(id_elem, id_token);

        fn is_param_node(node: Tree<FileElement<Lexem>>) -> bool {
            match node.borrow().get_value().element {
                Lexem::NonTerminal(NonTerminal::IdentList)
                | Lexem::NonTerminal(NonTerminal::IdentListIdent) => true,
                _ => false,
            }
        }

        let params: Vec<Param> = parse_list_filter(
            root.borrow().get_children()[3].clone(),
            Param::from,
            is_param_node,
        );

        let mut block = Block::from(root.borrow().get_children()[6].clone());
        let mut add_return = false;
        if let Some(last_statement) = block.statements.last() {
            if !matches!(last_statement, Statement::Return(_)) {
                add_return = true;
            }
            // if last statement return ok
        } else {
            add_return = true;
        }
        if add_return {
            let factor = Factor {
                factor_type: Some(Type::None),
                kind: FactorKind::None(empty_file_elt!(Token::None)),
            };
            let expr = Expression {
                expr_type: Option::Some(Type::None),
                kind: ExpressionKind::Factor(factor),
            };
            block.statements.push(Statement::Return(expr));
        }

        let localization = FileElement {
            element: true,
            len: identifier.len,
            start_char: identifier.start_char,
            start_line: identifier.start_line,
            end_line: identifier.end_line,
        };

        return Def {
            identifier,
            params,
            block,
            localization,
        };
    }
}

impl Into<Tree<String>> for Def {
    fn into(self) -> Tree<String> {
        let f_type = match &self.block.symbol_table {
            None => String::from(""),
            Some(table) => get_symbol(&table, &self.identifier.element.id)
                .expect("No symbol in symbol table")
                .symbol_type
                .to_string(),
        };

        let root = Node::new(String::from(format!("Def {}", f_type)));

        root.borrow_mut().add_child(
            &root,
            Node::new(format!("Identifier {}", self.identifier.element.name)),
        );

        let params_root = Node::new(String::from("PARAMS"));

        for param in self.params {
            params_root.borrow_mut().add_child(&root, param.into());
        }

        root.borrow_mut().add_child(&root, params_root);

        root.borrow_mut().add_child(&root, self.block.into());

        return root;
    }
}

impl Localizable for Def {
    fn get_len(&self) -> usize {
        self.identifier.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.identifier.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.block.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.identifier.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.block.get_end_char()
    }
}
