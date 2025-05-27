use crate::smollib::register_smollib_names;
use crate::{common::types::FileElement, reader};

use super::token_table::TokenTable;
use crate::common::diagnostic::*;
use crate::common::types::{IdToken, Token};
use colored::Colorize;

pub fn get_operator_token(op: &str) -> Option<Token> {
    match op {
        // Operators
        "+" => Some(Token::Add),
        "-" => Some(Token::Sub),
        "*" => Some(Token::Mult),
        "//" => Some(Token::Div),
        "/" => Some(Token::FloatDiv),
        "%" => Some(Token::Mod),
        "=" => Some(Token::Assign),

        // Boolean
        "==" => Some(Token::Equal),
        "!=" => Some(Token::NotEqual),
        ">" => Some(Token::Greater),
        ">=" => Some(Token::GreaterEq),
        "<" => Some(Token::Less),
        "<=" => Some(Token::LessEq),

        // Program structure
        ":" => Some(Token::Sep),
        "," => Some(Token::Comma),
        "(" => Some(Token::OpenParenthesis),
        ")" => Some(Token::CloseParenthesis),
        "[" => Some(Token::OpenBracket),
        "]" => Some(Token::CloseBracket),

        _ => None,
    }
}
pub struct Lexer {
    reader: reader::Reader,

    /// Pile contenant les nombres d'indentation
    indentation_stack: Vec<u64>,

    /// Nombre de tokens END à émettre avant de lire la suite du fichier
    end_token_count: u64,

    /// Vrai si le prochain token à émettre est un token BEGIN
    emit_begin: bool,

    /// Caractère courant
    peek: Option<char>,

    /// Vrai une fois que le token EOF a été émis
    emmitted_eof: bool,

    line_num: usize,
    char_num: usize,

    token_table: TokenTable,

    nb_errors: usize,
    nb_warnings: usize,

    diagnostics: Vec<Diagnostic>,
}

fn init_token_table() -> TokenTable {
    let mut table = TokenTable::new();

    table.reserve_word("True", Token::True);
    table.reserve_word("False", Token::False);
    table.reserve_word("None", Token::None);
    table.reserve_word("or", Token::Or);
    table.reserve_word("and", Token::And);
    table.reserve_word("not", Token::Not);
    table.reserve_word("if", Token::If);
    table.reserve_word("else", Token::Else);
    table.reserve_word("for", Token::For);
    table.reserve_word("while", Token::While);
    table.reserve_word("in", Token::In);
    table.reserve_word("def", Token::Def);
    table.reserve_word("return", Token::Return);
    table.reserve_word("print", Token::Print);
    table.reserve_word("println", Token::Println);

    register_smollib_names(&mut table);

    return table;
}

impl Lexer {
    pub fn new(mut reader: reader::Reader) -> Lexer {
        // Initialise la pile d'indentation avec 0 pour avoir le niveau global
        let indentation_stack = Vec::from([0]);
        let peek = reader.next();
        return Lexer {
            reader,
            indentation_stack,
            peek,
            emmitted_eof: false,
            line_num: 1,
            char_num: 1,
            end_token_count: 0,
            emit_begin: false,
            token_table: init_token_table(),
            nb_errors: 0,
            nb_warnings: 0,
            diagnostics: Vec::new(),
        };
    }

    pub fn get_nb_errors(&self) -> usize {
        return self.nb_errors;
    }

    fn construct_file_elem(&self, token: Token) -> FileElement<Token> {
        let calc_len = match token {
            Token::Identifier(IdToken { id, name: _ }) => self.token_table.get_ident_name(id).len(),
            Token::String(ref s) => s.len() + 2,
            Token::Newline => 0,
            Token::EOF => 0,
            Token::Begin => 0,
            Token::End => 0,
            _ => token.repr().len(),
        };
        FileElement {
            start_line: self.line_num,
            end_line: self.line_num,
            start_char: self.char_num - calc_len,
            len: calc_len,
            element: token,
        }
    }

    // Lit les caractères correspondant à des espaces et à des commentaires
    // Si une nouvelle ligne est rencontrée, renvoie true, et le nombre d'espaces depuis le début de la
    // dernière ligne lue. Sinon, renvoie false et le nombre d'espaces lu.
    fn skip_whitespace_and_comments(&mut self) -> (bool, u64) {
        let mut nb_spaces = 0;
        let mut new_line = false;

        loop {
            match self.peek {
                Some('\n') => {
                    new_line = true;
                    nb_spaces = 0;
                    self.line_num += 1;
                    self.char_num = 0;
                }
                Some(' ') => nb_spaces += 1,
                Some(c) if c.is_whitespace() => (),
                Some('#') => loop {
                    match self.read_next_char() {
                        Some('\n') | None => {
                            new_line = true;
                            nb_spaces = 0;
                            self.line_num += 1;
                            self.char_num = 0;
                            break;
                        }
                        _ => (),
                    }
                },
                _ => return (new_line, nb_spaces),
            }
            self.read_next_char();
        }
    }

    /// Consomme les caractères restants jusqu'au prochain caractère blanc,
    /// et renvoie le nombre de caractères lus.
    ///
    /// Permet de finir la lecture du token actuel en cas d'erreur
    fn skip_current_token(&mut self) -> usize {
        let mut nb_read = 0;

        while self.peek.is_some_and(|c| !c.is_whitespace()) {
            nb_read += 1;
            self.read_next_char();
        }
        return nb_read;
    }

    /*
       Permet de gérer l'émission des tokens BEGIN et END.
       S'il faut émettre un token BEGIN, passe `self.emit_begin` à true.
       S'il faut émettre des tokens END, met à jour `end_token_count`
       avec le nombre de tokens END restant à émettre.
    */
    fn parse_indentation(&mut self, indentation_number: u64) {
        let top: u64 = *self.indentation_stack.last().unwrap_or(&0);

        if indentation_number == top {
            return;
        } else if indentation_number > top {
            self.indentation_stack.push(indentation_number);
            self.emit_begin = true;
        } else {
            // Dépile jusqu'à trouver le bon numéro d'indentation
            loop {
                match self.indentation_stack.pop() {
                    Some(m) if m > indentation_number => self.end_token_count += 1,
                    Some(m) if m == indentation_number => {
                        self.indentation_stack.push(indentation_number);
                        break;
                    }
                    opt => {
                        let error_str = match opt {
                            Some(n) => format!("Expected {} spaces, got {} instead", n, indentation_number),
                            None => format!("Could not find previous block of indentation {}, maybe another IndentationError occured earlier in the program ?", indentation_number),
                        };
                        let diag = Diagnostic::new(
                            DiagnosticGravity::Error,
                            "IndentationError :".to_string(),
                            self.line_num,
                            self.line_num,
                            0,
                            self.char_num - 1,
                            error_str,
                        );
                        diag.display();
                        self.diagnostics.push(diag);
                        self.nb_errors += 1;
                        break;
                    }
                }
            }
        }
    }

    /// Lit le caractère suivant et le stocke dans self.peek
    fn read_next_char(&mut self) -> Option<char> {
        self.peek = self.reader.next();
        self.char_num += 1;
        return self.peek;
    }

    // Parse un integer a partir du caractère courant
    fn parse_integer(&mut self) -> Result<FileElement<Token>, Diagnostic> {
        let mut number: u64 = 0;

        let starting_char = self.char_num;

        match self.peek {
            Some('0') => {
                self.read_next_char();
                return Ok(self.construct_file_elem(Token::integer(0)));
            }
            Some(c) if c.is_digit(10) => {}

            // Ici on doit panic car la fonction parse_integer a été appelée par erreur du développeur et non
            // a cause d'une erreur lexicale
            _ => panic!("trying to use parse_integer while not on a digit"),
        }

        let mut overflow = false;

        while self.peek.is_some_and(|c| c.is_digit(10)) {
            let v: u64 = self.peek.unwrap().to_digit(10).unwrap().into();

            // Calcule la valeur de l'entier en vérifiant qu'elle peut
            // être contenue dans un entier machine
            // n = 10 * n + v
            let (n, of1) = number.overflowing_mul(10);
            let (n, of2) = n.overflowing_add(v);

            overflow = overflow | of1 | of2;
            number = n;
            self.read_next_char();
        }

        if overflow {
            let diag = Diagnostic::new(
                DiagnosticGravity::Error,
                "IntOverflow :".to_string(),
                self.line_num,
                self.line_num,
                starting_char,
                self.char_num - 1,
                "Integer cannot be represented on a 64 bit integer".to_string(),
            );
            diag.display();
            self.diagnostics.push(diag);
            self.nb_errors += 1;
            number = 0;
            //return Err(diag);
        }

        // Parse floats
        if let Some('.') = self.peek {
            self.read_next_char();
            return self.parse_float(number);
        }

        return Ok(self.construct_file_elem(Token::integer(number)));
    }

    // Parse la partie fractionnaire d'un float à partir du caractère courant
    fn parse_float(&mut self, int_part: u64) -> Result<FileElement<Token>, Diagnostic> {
        let mut number: f64 = int_part as f64;

        let starting_char = self.char_num;

        match self.peek {
            Some(c) if c.is_digit(10) => {}

            // Ici on doit panic car la fonction parse_integer a été appelée par erreur du développeur et non
            // a cause d'une erreur lexicale
            _ => panic!("trying to use parse_integer while not on a digit"),
        }

        let mut pow = 10;

        while self.peek.is_some_and(|c| c.is_digit(10)) {
            let v: f64 = self.peek.unwrap().to_digit(10).unwrap().into();

            // Calcule la valeur de l'entier en vérifiant qu'elle peut
            // être contenue dans un entier machine
            // n = 10 * n + v
            let v = v / pow as f64;
            let n = number + v;

            number = n;
            pow *= 10;
            self.read_next_char();
        }

        if f64::is_nan(number) {
            let diag = Diagnostic::new(
                DiagnosticGravity::Warning,
                "FloatOverflow :".to_string(),
                self.line_num,
                self.line_num,
                starting_char,
                self.char_num - 1,
                "Float cannot be represented on a 64 bit float, it will be represented as NaN"
                    .to_string(),
            );
            diag.display();
            self.diagnostics.push(diag);
            self.nb_warnings += 1;
        } else if f64::is_infinite(number) {
            let diag = Diagnostic::new(
                DiagnosticGravity::Warning,
                "FloatOverflow :".to_string(),
                self.line_num,
                self.line_num,
                starting_char,
                self.char_num - 1,
                format!(
                    "Value cannot be represented as a finite float, it will be represented as {}",
                    number
                ),
            );
            diag.display();
            self.diagnostics.push(diag);
            self.nb_warnings += 1;
            number = 0.;
        }

        return Ok(self.construct_file_elem(Token::float(number)));
    }

    // Parse un identifier à partir du caractère courant
    // Ne vérifie pas que le premier caractère n'est pas un chiffre
    fn parse_identifier(&mut self) -> FileElement<Token> {
        let mut identifier = String::new();

        while self
            .peek
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            identifier.push(self.peek.unwrap());
            self.read_next_char();
        }

        let token = self.token_table.get_token(identifier);

        return self.construct_file_elem(token);
    }

    // Parse un string à partir du caractère courant.
    // Ne vérifie pas que le caractère courant est un '"'
    fn parse_string(&mut self) -> Result<FileElement<Token>, Vec<Diagnostic>> {
        let mut text = String::new();

        let mut diags: Vec<Diagnostic> = Vec::new();

        let start_char = self.char_num;

        self.read_next_char();

        loop {
            match self.peek {
                Some('"') => {
                    self.read_next_char();
                    break;
                }
                Some('\\') => {
                    self.read_next_char();
                    match self.peek {
                        Some('"') => text.push('"'),
                        Some('\\') => text.push('\\'),
                        Some('n') => text.push('\n'),
                        Some('e') => text.push('\x1b'),
                        other => {
                            let diag = Diagnostic::new(
                                DiagnosticGravity::Error,
                                "InvalidEscapeSequence :".to_string(),
                                self.line_num,
                                self.line_num,
                                self.char_num - 1,
                                self.char_num,
                                format!(
                                    "\\{} unkown, expected after \\ : '{}', '{}' or '{}'",
                                    other.unwrap().to_string().truecolor(255, 0, 0),
                                    "\"".truecolor(0, 255, 0),
                                    "\\".truecolor(0, 255, 0),
                                    "n".truecolor(0, 255, 0)
                                ),
                            );
                            diags.push(diag);
                        }
                    }
                }
                Some('\n') => {
                    let diag = Diagnostic::new(
                        DiagnosticGravity::Error,
                        "UnterminatedString :".to_string(),
                        self.line_num,
                        self.line_num,
                        start_char,
                        self.char_num,
                        format!("String must be terminated by {}", "\"".truecolor(0, 255, 0)),
                    );
                    diags.push(diag);
                    break;
                }
                Some(c) => text.push(c),
                None => break,
            }
            self.read_next_char();
        }

        if !diags.is_empty() {
            return Err(diags);
        }

        return Ok(self.construct_file_elem(Token::String(text)));
    }

    /// Tente une opération de parsing qui peut échouer en émettant un diagnostic.
    /// Si un diagnostic est émis, l'affiche et tente de renvoyer le token suivant.
    fn try_parse(
        &mut self,
        result: Result<FileElement<Token>, Diagnostic>,
    ) -> Option<FileElement<Token>> {
        return self.try_parse_multiple(result.map_err(|e| vec![e]));
    }

    /// Tente une opération de parsing qui peut échouer en émettant plusieurs diagnostics.
    /// Si des diagnostics sont émis, les affiche et tente de renvoyer le token suivant.
    fn try_parse_multiple(
        &mut self,
        result: Result<FileElement<Token>, Vec<Diagnostic>>,
    ) -> Option<FileElement<Token>> {
        match result {
            Ok(elem) => Some(elem),
            Err(diags) => {
                for diag in diags {
                    diag.display();
                    match &diag.gravity {
                        DiagnosticGravity::Warning => self.nb_warnings += 1,
                        DiagnosticGravity::Error => self.nb_errors += 1,
                    }
                    self.diagnostics.push(diag);
                }
                return self.next();
            }
        }
    }

    /// Skip le token actuel en émettant l'erreur donnée, et passe au token suivant
    /// Considère que le token se trouve sur la ligne actuelle du lexer, commence
    /// au caractère `start_char` et se termine après l'appel à `self::skip_current_token`.
    fn skip_with_error(
        &mut self,
        error: String,
        msg: String,
        start_char: usize,
    ) -> Option<FileElement<Token>> {
        self.skip_current_token();

        let diag = Diagnostic::new(
            DiagnosticGravity::Error,
            error,
            self.line_num,
            self.line_num,
            start_char,
            self.char_num - 1,
            msg,
        );

        diag.display();
        self.diagnostics.push(diag);
        self.nb_errors += 1;

        return self.next();
    }
}

impl Iterator for Lexer {
    type Item = FileElement<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! operator_file_elem {
            ($s: expr) => {
                match get_operator_token($s) {
                    None => None,
                    Some(token) => Some(self.construct_file_elem(token)),
                }
            };
        }

        macro_rules! try_parse {
            ($parse: expr) => {{
                let res = $parse;
                self.try_parse(res)
            }};
        }

        // Handle BEGIN and END tokens

        if self.emit_begin {
            self.emit_begin = false;
            return Some(self.construct_file_elem(Token::Begin));
        }

        if self.end_token_count > 0 {
            self.end_token_count -= 1;
            return Some(self.construct_file_elem(Token::End));
        }

        // Already at the end of the file
        if self.peek.is_none() {
            if self.emmitted_eof {
                return None;
            } else {
                self.emmitted_eof = true;
                return Some(self.construct_file_elem(Token::EOF));
            }
        }

        // Skip whitespace and comments
        let (new_line, nb_indentation) = self.skip_whitespace_and_comments();

        // Handle BEGIN and END tokens if indentation changes on a new line
        if new_line {
            self.parse_indentation(nb_indentation);
            return Some(self.construct_file_elem(Token::Newline));
        }

        match self.peek {
            // <integer>
            Some(c) if c.is_digit(10) => return try_parse!(self.parse_integer()),

            // <identifier>
            Some(c) if c.is_ascii_alphabetic() || c == '_' => return Some(self.parse_identifier()),

            // <string>
            Some('"') => {
                let res = self.parse_string();
                return self.try_parse_multiple(res);
            }

            // !=
            Some('!') => match self.read_next_char() {
                Some('=') => {
                    self.read_next_char();
                    return operator_file_elem!("!=");
                }
                _ => {
                    return self.skip_with_error(
                        String::from("InvalidToken"),
                        format!(
                            "Unrecognized token, did you mean {} ?",
                            "!=".truecolor(0, 255, 0)
                        ),
                        self.char_num - 1,
                    )
                }
            },

            // //
            Some('/') => match self.read_next_char() {
                Some('/') => {
                    self.read_next_char();
                    return operator_file_elem!("//");
                }
                _ => return operator_file_elem!("/"),
            },

            // <, >, =, <=, >=, ==
            Some(c) if c == '<' || c == '>' || c == '=' => match self.read_next_char() {
                Some('=') => {
                    self.read_next_char();
                    return operator_file_elem!(format!("{}=", c).as_str());
                }
                _ => return operator_file_elem!(c.to_string().as_str()),
            },

            // Tokens à 1 caractère
            Some(c) => {
                self.read_next_char();
                match get_operator_token(c.to_string().as_str()) {
                    Some(token) => return Some(self.construct_file_elem(token)),
                    None => {
                        return self.skip_with_error(
                            String::from("InvalidToken"),
                            format!(
                                "Unrecognized token '{}'",
                                c.to_string().truecolor(255, 0, 0)
                            ),
                            self.char_num - 1,
                        )
                    }
                }
            }
            None => {
                self.emmitted_eof = true;
                return Some(self.construct_file_elem(Token::EOF));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::types::Token,
        lexer::lexer::{get_operator_token, init_token_table, Lexer},
        reader::Reader,
    };

    #[test]
    fn test_eof() {
        let mut lexer = Lexer::new(Reader::from(""));
        let n = lexer.next();
        assert_ne!(None, n);
        let t = n.unwrap().element;
        assert!(t == Token::EOF);
        assert!(lexer.next() == None);
    }

    #[test]
    fn test_integer() {
        let mut lexer = Lexer::new(Reader::from("123"));
        assert!(lexer.next().unwrap().element == Token::integer(123));
        assert!(lexer.next().unwrap().element == Token::EOF);
        assert!(lexer.next() == None);
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new(Reader::from("hello"));
        let mut tokentable = init_token_table();
        let token = tokentable.get_token("hello".to_string());
        assert!(lexer.next().unwrap().element == token);
        assert!(lexer.next().unwrap().element == Token::EOF);
        assert!(lexer.next() == None);
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new(Reader::from("\"hello\""));
        assert!(lexer.next().unwrap().element == Token::String("hello".to_string()));
        assert!(lexer.next().unwrap().element == Token::EOF);
        assert!(lexer.next() == None);
    }

    #[test]
    fn test_tokens() {
        let mut lexer = Lexer::new(Reader::from("+ - * // % == = != < > <= >= True False None and or not def for in if else return print , : ( ) [ ]"));
        let mut tokentable = init_token_table();
        assert!(lexer.next().unwrap().element == get_operator_token("+").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("-").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("*").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("//").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("%").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("==").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("=").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("!=").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("<").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token(">").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("<=").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token(">=").unwrap());
        assert!(lexer.next().unwrap().element == tokentable.get_token("True".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("False".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("None".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("and".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("or".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("not".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("def".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("for".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("in".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("if".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("else".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("return".to_string()));
        assert!(lexer.next().unwrap().element == tokentable.get_token("print".to_string()));
        assert!(lexer.next().unwrap().element == get_operator_token(",").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token(":").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("(").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token(")").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("[").unwrap());
        assert!(lexer.next().unwrap().element == get_operator_token("]").unwrap());
        assert!(lexer.next().unwrap().element == Token::EOF);
        assert!(lexer.next() == None);
    }
}
