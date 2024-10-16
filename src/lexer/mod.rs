use crate::reader;

// TODO: Remove
// Macro temporaire pour faire des tokens a partir d'un string
macro_rules! tokenify {
    ($($arg:tt)*) => {
        Some(String::from(format!($($arg)*)))
    };
}

pub fn get_operator_token(op: &str) -> Option<Token> {
    match op {
        "+" => tokenify!("<+>"),
        "-" => tokenify!("<->"),
        "*" => tokenify!("<*>"),
        "%" => tokenify!("<%>"),
        ":" => tokenify!("<:>"),
        "[" => tokenify!("<[>"),
        "]" => tokenify!("<]>"),
        "(" => tokenify!("<(>"),
        ")" => tokenify!("<)>"),
        "," => tokenify!("<,>"),
        ">" => tokenify!("<gt>"),
        "<" => tokenify!("<lt>"),
        ">=" => tokenify!("<ge>"),
        "<=" => tokenify!("<le>"),
        "==" => tokenify!("<eq>"),
        "=" => tokenify!("<=>"),
        "!=" => tokenify!("<ne>"),
        "//" => tokenify!("<//>"),
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
}

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
    };
}

type Token = String;
impl Lexer {

    fn syntax_error(&self, err: &str) {
        panic!("Syntax Error l.{}:{} {}", self.line_num, self.char_num, err);
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
                },
                Some(' ') => nb_spaces += 1,
                Some(c) if c.is_whitespace() => (),
                Some('#') => {
                    loop {
                        match self.read_next_char() {
                            Some('\n') | None => {
                                new_line = true;
                                nb_spaces = 0;
                                self.line_num += 1;
                                self.char_num = 0;
                                break;
                            },
                            _ => (),
                        }
                    }
                }
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
    fn parse_indentation(&mut self, indentation_number: u64){

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
                    _ => self.syntax_error(format!("Indentation error (expected {})", indentation_number).as_str()),
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
    fn parse_integer(&mut self) -> Token {

        let mut number: usize = 0;

        match self.peek {
            Some('0') => {
                self.read_next_char();
                return format!("<int, 0>");
            },
            Some(c) if c.is_digit(10) => {},
            _ => panic!("trying to use parse_integer while not on a digit"),
        }

        while self.peek.is_some_and(|c| c.is_digit(10)) {
            let v: usize = match self.peek.unwrap().to_digit(10) {
                Some(v) => v.try_into().unwrap(),
                None => {
                    self.syntax_error("Could not convert digit to usize");
                    break;
                }
            };

            // Calcule la valeur de l'entier en vérifiant qu'elle peut
            // être contenue dans un entier machine
            // n = 10 * n + v
            let (n, of1) = number.overflowing_mul(10);
            let (n, of2) = n.overflowing_add(v);

            if of1 | of2 {
                self.syntax_error("Integer cannot be represented on this machine");
            }
            number = n;

            self.read_next_char();
        }

        return format!("<int, {}>", number);
    }

    // Parse un identifier à partir du caractère courant
    // Ne vérifie pas que le premier caractère n'est pas un chiffre
    fn parse_identifier(&mut self) -> Token {

        let mut identifier = String::new();

        while self.peek.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            identifier.push(self.peek.unwrap());
            self.read_next_char();
        }

        return format!("<ident, \"{}\">", identifier);

    }

    // Parse un string à partir du caractère courant.
    // Ne vérifie pas que le caractère courant est un '"'
    fn parse_string(&mut self) -> Token {

        let mut text = String::new();

        self.read_next_char();

        loop {
            match self.peek {
                Some('"') => {self.read_next_char(); break;},
                Some('\\') => {
                    self.read_next_char();
                    match self.peek {
                        Some('"') => text.push('"'),
                        Some('\\') => text.push('\\'),
                        Some('n') => text.push('\n'),
                        _ => {
                            self.syntax_error("Invalid escape sequence");
                            break;
                        }
                    }
                },
                Some('\n') => {self.syntax_error("Unterminated string");},
                Some(c) => text.push(c),
                None => break,
            }
            self.read_next_char();
        }

        return format!("<str, \"{}\">", text);
    }
}


impl Iterator for Lexer {
    // TODO: Change to token type
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {

        // Handle BEGIN and END tokens

        if self.emit_begin {
            self.emit_begin = false;
            return tokenify!("BEGIN");
        }

        if self.end_token_count > 0 {
            self.end_token_count -= 1;
            return tokenify!("END");
        }

        // Already at the end of the file
        if self.peek.is_none() {
            if self.emmitted_eof {
                return None;
            } else {
                self.emmitted_eof = true;
                return tokenify!("<EOF>");
            }
        }

        // Skip whitespace and comments
        let (new_line, nb_indentation) = self.skip_whitespace_and_comments();
        
        // Handle BEGIN and END tokens if indentation changes on a new line
        if new_line {
            self.parse_indentation(nb_indentation);
            return tokenify!("NEWLINE");
        }

        match self.peek {

            // <integer>
            Some(c) if c.is_digit(10) => return Some(self.parse_integer()),

            // <identifier>
            Some(c) if c.is_ascii_alphabetic() || c == '_' => return Some(self.parse_identifier()),

            // <string>
            Some('"') => return Some(self.parse_string()),

            // !=
            Some('!') => {
                match self.read_next_char() {
                    Some('=') => {
                        self.read_next_char();
                        return get_operator_token("!=")
                    },
                    _ => self.syntax_error("Invalid token"),
                }
            }

            // //
            Some('/') => {
                match self.read_next_char() {
                    Some('/') => {
                        self.read_next_char();
                        return get_operator_token("//");
                    },
                    _ => self.syntax_error("Invalid token"),
                }
            }

            // <, >, =, <=, >=, ==
            Some(c) if c == '<' || c == '>' || c == '=' => {
                match self.read_next_char() {
                    Some('=') => {
                        self.read_next_char();
                        return get_operator_token(format!("{}=",c).as_str())
                    },
                    _ => return get_operator_token(c.to_string().as_str()),
                }
            }

            // Tokens à 1 caractère
            Some(c) => {
                self.read_next_char();
                match get_operator_token(c.to_string().as_str()) {
                    Some(token) => return Some(token),
                    None => self.syntax_error("Invalid token"),
                }
            },
            None => {
                self.emmitted_eof = true;
                return tokenify!("<EOF>");
            }
        }

        return None;
    }
}