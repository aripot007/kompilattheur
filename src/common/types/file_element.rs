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

    /// La ligne 
    pub line: u64,

    /// Le numéro du caractère de début
    pub start_char: u64,

    /// La longueur de l'élément en caractères
    pub len: usize,

    /// L'élément correspondant
    pub element: T,
}

/// Special FileElement for EOF
pub const EOF: FileElement<Token> = FileElement {
    line: 0,
    start_char: 0,
    len: 0,
    element: Token::EOF,
};

impl<T> PartialEq for FileElement<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<T> Eq for FileElement<T> where T: Eq {

}
