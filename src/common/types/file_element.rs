use std::fmt::Display;

use crate::common::localizable::Localizable;

use super::Token;

/// Représente un élément du code source ayant une position et une longueur dans le fichier (token, suite de tokens ...)
///
/// L'égalité ne compare que les éléments
///
/// ```
/// let e1 = FileElement{line:0, start_char: 0, len: 0, element: String::from("Hello")};
/// let e2 = FileElement{line:42, start_char: 42, len: 42, element: String::from("Hello")};
/// assert_eq!(e1, e2);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct FileElement<T> {
    /// La ligne de départ
    pub start_line: usize,

    /// La ligne de fin
    pub end_line: usize,

    /// Le numéro du caractère de début
    pub start_char: usize,

    /// La longueur de l'élément en caractères
    pub len: usize,

    /// L'élément correspondant
    pub element: T,
}

/// Special FileElement for EOF
pub const EOF: FileElement<Token> = FileElement {
    start_line: 0,
    end_line: 0,
    start_char: 0,
    len: 0,
    element: Token::EOF,
};

impl<T> Localizable for FileElement<T> {
    fn get_len(&self) -> usize {
        self.len
    }

    fn get_start_line(&self) -> usize {
        self.start_line
    }

    fn get_end_line(&self) -> usize {
        self.start_line
    }

    fn get_start_char(&self) -> usize {
        self.start_char
    }

    fn get_end_char(&self) -> usize {
        self.start_char + self.len
    }
}

impl<T> PartialEq for FileElement<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<T> Eq for FileElement<T> where T: Eq {}

impl<T: Display> Display for FileElement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.element.fmt(f)
    }
}

/// Permet de recréer un FileElement en gardant les informations mais en changeant l'élément
macro_rules! file_element_from {
    ($from: expr, $new_val: expr) => {
        FileElement {
            len: $from.len,
            start_line: $from.start_line,
            end_line: $from.end_line,
            start_char: $from.start_char,
            element: $new_val,
        }
    };
}

pub(crate) use file_element_from;

macro_rules! empty_file_elt {
    ($elt: expr) => {
        FileElement {
            len: 0,
            start_line: 0,
            end_line: 0,
            start_char: 0,
            element: $elt,
        }
    };
}

pub(crate) use empty_file_elt;

#[cfg(test)]
mod tests {
    use super::FileElement;

    #[test]
    fn test_file_elem_from() {
        let e1 = FileElement {
            len: 0,
            start_line: 1,
            end_line: 1,
            start_char: 42,
            element: 123456,
        };

        let e2 = file_element_from!(e1, "chokbar");

        assert_eq!(e1.len, e2.len);
        assert_eq!(e1.start_line, e2.start_line);
        assert_eq!(e1.end_line, e2.end_line);
        assert_eq!(e1.start_char, e2.start_char);
        assert_eq!(e1.element, 123456);
        assert_eq!(e2.element, "chokbar");
    }
}
