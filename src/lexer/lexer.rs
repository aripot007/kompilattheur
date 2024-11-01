use crate::{common::types::FileElement, reader};

use super::token_table::TokenTable;
use crate::common::types::{Token, IdToken};
use colored::Colorize;
use crate::common::diagnostic::*;

pub fn get_operator_token(op: &str) -> Option<Token> {
    match op {
        // Operators
        "+" => Some(Token::Add),
        "-" => Some(Token::Sub),
        "*" => Some(Token::Mult),
        "//" => Some(Token::Div),
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

    // Pile contenant les nombres d'indentation
    indentation_stack: Vec<u64>,

    // Nombre de tokens END à émettre avant de lire la suite du fichier
    end_token_count: u64,

    // Vrai si le prochain token à émettre est un token BEGIN
    emit_begin: bool,

    // Caractère courant
    peek: Option<char>,

    // Vrai une fois que le token EOF a été émis
    emmitted_eof: bool,

    line_num: u64,
    char_num: u64,

    token_table: TokenTable,
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
    table.reserve_word("in", Token::In);
    table.reserve_word("def", Token::Def);
    table.reserve_word("return", Token::Return);
    table.reserve_word("print", Token::Print);

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
        };
    }

    fn construct_file_elem(&self, token: Token) -> FileElement<Token> {
        FileElement {
            line: self.line_num,
            start_char: self.char_num,
            len: match token {
                Token::Identifier(IdToken{id}) => self.token_table.get_ident_name(id).len(),
                _ => token.repr().len()
            },
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
                    Some(m) if m < indentation_number => (),
                    Some(m) if m == indentation_number => {
                        self.indentation_stack.push(indentation_number);
                        break;
                    }
                    _ => Diagnostic::new(
                        DiagnosticGravity::Error,
                        "IdentationError :".to_string(),
                        self.line_num,
                        self.line_num,
                        self.char_num,
                        self.char_num,
                        format!("Expected {})", indentation_number),
                    )
                    .display(),
                }
            }
        }
    }

    // Lit le caractère suivant et le stocke dans self.peek
    fn read_next_char(&mut self) -> Option<char> {
        self.peek = self.reader.next();
        self.char_num += 1;
        return self.peek;
    }

    // Parse un integer a partir du caractère courant
    fn parse_integer(&mut self) -> FileElement<Token> {

        let mut number: u64 = 0;

        match self.peek {
            Some('0') => {
                self.read_next_char();
                return self.construct_file_elem(Token::integer(0));
            },
            Some(c) if c.is_digit(10) => {},

            // Ici on doit panic car la fonction parse_integer a été appelée par erreur du développeur et non
            // a cause d'une erreur lexicale
            _ => panic!("trying to use parse_integer while not on a digit"),
        }

        while self.peek.is_some_and(|c| c.is_digit(10)) {
            let v: u64 = match self.peek.unwrap().to_digit(10) {
                Some(v) => v.try_into().unwrap(),
                None => {
                    Diagnostic::new(
                        DiagnosticGravity::Error,
                        "IntOverflow :".to_string(),
                        self.line_num,
                        self.line_num,
                        self.char_num,
                        self.char_num,
                        "Integer cannot be represented on this machine".to_string(),
                    )
                    .display();
                    break;
                }
            };

            // Calcule la valeur de l'entier en vérifiant qu'elle peut
            // être contenue dans un entier machine
            // n = 10 * n + v
            let (n, of1) = number.overflowing_mul(10);
            let (n, of2) = n.overflowing_add(v);

            if of1 | of2 {
                Diagnostic::new(
                    DiagnosticGravity::Error,
                    "IntOverflow :".to_string(),
                    self.line_num,
                    self.line_num,
                    self.char_num,
                    self.char_num+1,
                    "Integer cannot be represented on a 64 bit integer".to_string(),
                )
                .display();
                panic!("Integer cannot be represented on a 64 bit integer")
            }
            number = n;

            self.read_next_char();
        }

        return self.construct_file_elem(Token::integer(number));
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
    fn parse_string(&mut self) -> FileElement<Token> {

        let mut text = String::new();

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
                        _ => {
                            Diagnostic::new(
                                DiagnosticGravity::Error,
                                "InvalidEscapeSequence :".to_string(),
                                self.line_num,
                                self.line_num,
                                self.char_num,
                                self.char_num+1,
                                "Expected after \\ : '\"', '\\' or 'n'".to_string(),
                            )
                            .display();
                            break;
                        }
                    }
                }
                Some('\n') => Diagnostic::new(
                    DiagnosticGravity::Error,
                    "UnterminatedString :".to_string(),
                    self.line_num,
                    self.line_num,
                    self.char_num,
                    self.char_num+1,
                    "String must be terminated by '\"'".to_string(),
                )
                .display(),
                Some(c) => text.push(c),
                None => break,
            }
            self.read_next_char();
        }

        return self.construct_file_elem(Token::String(text));
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
            Some(c) if c.is_digit(10) => return Some(self.parse_integer()),

            // <identifier>
            Some(c) if c.is_ascii_alphabetic() || c == '_' => return Some(self.parse_identifier()),

            // <string>
            Some('"') => return Some(self.parse_string()),

            // !=
            Some('!') => match self.read_next_char() {
                Some('=') => {
                    self.read_next_char();
                        return operator_file_elem!("!=")
                }
                other => Diagnostic::new(
                    DiagnosticGravity::Error,
                    "InvalidToken :".to_string(),
                    self.line_num,
                    self.line_num,
                    self.char_num,
                    self.char_num+1,
                    format!("{} is invalid, did you mean {} ?",format!("={}", other.unwrap()).truecolor(255, 0, 0), "!=".truecolor(0, 255, 0)),
                )
                .display(),
            },

            // //
            Some('/') => match self.read_next_char() {
                Some('/') => {
                    self.read_next_char();
                        return operator_file_elem!("//");
                }
                other => Diagnostic::new(
                    DiagnosticGravity::Error,
                    "InvalidToken :".to_string(),
                    self.line_num,
                    self.line_num,
                    self.char_num,
                    self.char_num+1,
                    format!("{} is invalid, did you mean {} ?",format!("/{}", other.unwrap()).truecolor(255, 0, 0), "//".truecolor(0, 255, 0)),
                )
                .display(),
            },

            // <, >, =, <=, >=, ==
            Some(c) if c == '<' || c == '>' || c == '=' => match self.read_next_char() {
                Some('=') => {
                    self.read_next_char();
                    return operator_file_elem!(format!("{}=",c).as_str())
                }
                _ => return operator_file_elem!(c.to_string().as_str()),
            },

            // Tokens à 1 caractère
            Some(c) => {
                self.read_next_char();
                match get_operator_token(c.to_string().as_str()) {
                    Some(token) => return Some(self.construct_file_elem(token)),
                    None => Diagnostic::new(
                        DiagnosticGravity::Error,
                        "InvalidToken :".to_string(),
                        self.line_num,
                        self.line_num,
                        self.char_num,
                        self.char_num+1,
                        format!("{} is invalid", c.to_string().truecolor(255, 0, 0)),
                    )
                    .display(),
                }
            }
            None => {
                self.emmitted_eof = true;
                return Some(self.construct_file_elem(Token::EOF));
            }
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::types::Token, lexer::lexer::Lexer, reader::Reader};

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

        // TODO: Adapter le test pour éviter de panic 
        let mut lexer = Lexer::new(Reader::from("123"));
        assert!(lexer.next().unwrap().element == Token::integer(123));
        assert!(lexer.next().unwrap().element == Token::EOF);
        assert!(lexer.next() == None);
    }
    
    #[test]
    #[should_panic]
    fn test_int_overflow() {
        let mut lexer = Lexer::new(Reader::from("18446744073709551616"));
        lexer.next();
    }
    
    
}
