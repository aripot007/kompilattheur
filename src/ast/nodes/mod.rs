mod factor;
use std::fmt::Display;

pub use factor::*;
mod defs;
pub use defs::*;
mod def;
pub use def::*;
mod root;
pub use root::*;
mod block;
pub use block::*;

use crate::{common::types::{FileElement, Tree}, parser::Lexem};

/// Transforme un arbre représentant une liste (arg list, statement list ...) en un vec
/// contenant les blocks de l'AST correspondant.
/// 
/// L'arbre représentant une liste doit respecter le schéma suivant :
/// Un noeud représentant la liste contient :
/// - soit un fils gauche avec un élément, et un noeud de liste en fils droit
/// - soit aucun fils, dans ce cas la liste est vide
/// 
/// parse_U est une fonction convertissant un Tree<T> en élément U, qui sera appelée sur les
/// noeuds représentant les éléments de la liste
pub (super) fn parse_list<T, U>(root: Tree<T>, parse_U: fn(Tree<T>) -> U) -> Vec<U> {
    let mut values: Vec<U> =  Vec::new();

    fn parse<T, U>(node: Tree<T>, values: &mut Vec<U>, parse_U: fn(Tree<T>) -> U) {

        let nb_children = node.borrow().childs.len();

        if nb_children == 0 {
            return;
        } else if nb_children != 2 {
            panic!("Malformed list tree : found {} children instead of 0 or 2.", nb_children);
        }

        values.push(parse_U(node.borrow().childs[0].clone()));

        parse(node.borrow().childs[1].clone(), values, parse_U);
    }

    parse(root, &mut values, parse_U);
    return values;
}

/// Un noeud d'ast doit pouvoir être créé depuis l'arbre concret correspondant, et
/// doit pouvoir être convertit en arbre de String représentant chaque noeud, pour
/// faciliter l'affichage.
trait AstNode: From<Tree<FileElement<Lexem>>> + Into<Tree<String>> {}
