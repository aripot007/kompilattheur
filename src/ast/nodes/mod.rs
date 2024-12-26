mod factor;
pub use factor::Factor;
mod defs;
pub use defs::Defs;
mod def;
pub use def::Def;
mod root;
pub use root::Root;
mod block;
pub use block::Block;
mod param;
pub use param::Param;
mod statement;
pub use statement::Statement;
mod expression;
pub use expression::Expression;
mod for_loop;
pub use for_loop::For;

use crate::{common::types::{FileElement, Tree}, parser::Lexem};

/// Transforme un arbre représentant une liste (arg list, statement list ...) en un vec
/// contenant les blocks de l'AST correspondant.
/// 
/// L'arbre représentant une liste doit respecter le schéma suivant :
/// Un noeud représentant la liste contient :
/// - soit un fils gauche avec un élément, et un noeud de liste en fils droit
/// - soit aucun fils, dans ce cas la liste est vide
/// 
/// parse_u est une fonction convertissant un Tree<T> en élément U, qui sera appelée sur les
/// noeuds représentant les éléments de la liste
pub (super) fn parse_list<T, U>(root: Tree<T>, parse_u: fn(Tree<T>) -> U) -> Vec<U> {
    return parse_list_filter(root, parse_u, |_| true);
}

/// Transforme un arbre représentant une liste (arg list, statement list ...) en un vec
/// contenant les blocks de l'AST correspondant, en appliquant un filtre.
/// 
/// L'arbre représentant une liste doit respecter le schéma suivant :
/// Un noeud représentant la liste contient :
/// - soit un fils gauche avec un élément, et un noeud de liste en fils droit
/// - soit aucun fils, dans ce cas la liste est vide
/// 
/// parse_u est une fonction convertissant un Tree<T> en élément U, qui sera appelée sur les
/// noeuds représentant les éléments de la liste pour lesquels filter_t renvoie True.
/// 
/// Les noeuds pour lesquels filter_t renvoient False sont ignorés.
pub (super) fn parse_list_filter<T, U>(root: Tree<T>, parse_u: fn(Tree<T>) -> U, filter_t: fn(Tree<T>) -> bool) -> Vec<U> {
    let mut values: Vec<U> =  Vec::new();

    fn parse<T, U>(node: Tree<T>, values: &mut Vec<U>, parse_u: fn(Tree<T>) -> U, filter_t: fn(Tree<T>) -> bool) {

        let nb_children = node.borrow().childs.len();

        if nb_children == 0 {
            return;
        } else if nb_children != 2 {
            panic!("Malformed list tree : found {} children instead of 0 or 2.", nb_children);
        }

        if filter_t(node.clone()) {
            values.push(parse_u(node.borrow().childs[0].clone()));
        }

        parse(node.borrow().childs[1].clone(), values, parse_u, filter_t);
    }

    parse(root, &mut values, parse_u, filter_t);
    return values;
}

/// Un noeud d'ast doit pouvoir être créé depuis l'arbre concret correspondant, et
/// doit pouvoir être convertit en arbre de String représentant chaque noeud, pour
/// faciliter l'affichage.
trait AstNode: From<Tree<FileElement<Lexem>>> + Into<Tree<String>> {}
