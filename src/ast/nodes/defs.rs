use super::{list_into_tree, parse_list, AstNode, Def};
use crate::{
    common::{
        localizable::Localizable,
        types::{FileElement, Node, Tree},
    },
    parser::Lexem,
};

pub struct Defs {
    defs: Vec<Def>,
    localization: FileElement<bool>,
}

impl AstNode for Defs {}

impl From<Tree<FileElement<Lexem>>> for Defs {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let defs: Vec<Def> = parse_list(root, Def::from);

        let localization = FileElement {
            element: true,
            len: defs
                .last()
                .map_or(0, |def| def.get_end_char())
                .saturating_sub(
                    // -
                    defs.first()
                        .map_or(0, |def| def.localization.get_start_char()),
                ),
            start_line: defs
                .first()
                .map_or(0, |def| def.localization.get_start_line()),
            end_line: defs.last().map_or(0, |def| def.localization.get_end_line()),
            start_char: defs
                .first()
                .map_or(0, |def| def.localization.get_start_char()),
        };

        return Defs { defs, localization };
    }
}

impl Into<Tree<String>> for Defs {
    fn into(self) -> Tree<String> {
        list_into_tree!("DEFS", self.defs)
    }
}

impl Localizable for Defs {
    fn get_len(&self) -> usize {
        self.localization.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.localization.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.localization.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.localization.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.localization.get_end_char()
    }
}
