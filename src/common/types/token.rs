use std::fmt::Display;


#[derive(Clone, Hash)]
pub struct NumToken {
    value: u64,
}

impl PartialEq for NumToken {
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}
impl Eq for NumToken {}


#[derive(Clone, Hash)]
pub struct IdToken {
    pub id: usize,
}

impl PartialEq for IdToken {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}
impl Eq for IdToken {}

#[derive(Clone, Hash)]
pub enum Token {
    Integer(NumToken),
    Identifier(IdToken),
    String(String),
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Equal,
    Assign,
    NotEqual,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Begin,
    End,
    Newline,
    EOF,
    Comma,
    Sep,
    OpenParenthesis,
    CloseParenthesis,
    OpenBracket,
    CloseBracket,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Def,
    For,
    If,
    Else,
    Return,
    Print,
}

impl Display for Token {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(_) => write!(f, "<Identifier, {}>", self.repr()),
            Token::String(_) => write!(f, "<String, {}", self.repr()),
            Token::Integer(_) => write!(f, "<Int, {}", self.repr()),
            _ => write!(f, "<{}>", self.repr()),
        }
        
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(num1), Self::Integer(num2)) => num1 == num2,
            (Self::Identifier(id1), Self::Identifier(id2)) => id1 == id2,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl Eq for Token {}

impl Token {

    pub fn is_same_type(&self, other: &Token) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }

    pub fn integer(value: u64) -> Token {
        Token::Integer(NumToken {value})
    }

    /// Renvoie la représentation de ce token dans le code source
    /// Pour les tokens simples, les strings, les entiers et les mots clés réservés, renvoie le texte correspondant dans le code source.
    /// Pour les identifier, renvoie l'id de l'identifier
    /// 
    /// ```
    /// assert_eq!(Token::Add.repr(), "+".to_string());
    /// assert_eq!(Token::integer(42).repr(), "42".to_string());
    /// assert_eq!(Token::String("Hello World !".to_string()).repr(), "\"Hello World !\"".to_string());
    /// assert_eq!(Token::Identifier(IdToken {id:42}), "42".to_string());
    /// ```
    pub fn repr(&self) -> String {
        match self {
            Token::Integer(num_token) => num_token.value.to_string(),
            Token::Identifier(id_token) => id_token.id.to_string(),
            Token::String(string) => format!("\"{}\"", string.escape_debug()),
            Token::Add => String::from("+"),
            Token::Sub => String::from("-"),
            Token::Mult => String::from("*"),
            Token::Div => String::from("//"),
            Token::Mod => String::from("%"),
            Token::Equal => String::from("=="),
            Token::Assign => String::from("="),
            Token::NotEqual => String::from("!="),
            Token::Less => String::from("<"),
            Token::Greater => String::from(">"),
            Token::LessEq => String::from("="),
            Token::GreaterEq => String::from(">="),
            Token::Begin => String::from("BEGIN"),
            Token::End => String::from("END"),
            Token::Newline => String::from("NEWLINE"),
            Token::EOF => String::from("EOF"),
            Token::True => String::from("True"),
            Token::False => String::from("False"),
            Token::None => String::from("None"),
            Token::And => String::from("and"),
            Token::Or => String::from("or"),
            Token::Not => String::from("not"),
            Token::Def => String::from("def"),
            Token::For => String::from("for"),
            Token::If => String::from("if"),
            Token::Else => String::from("else"),
            Token::Return => String::from("return"),
            Token::Print => String::from("print"),
            Token::Comma => String::from(","),
            Token::Sep => String::from(":"),
            Token::OpenParenthesis => String::from("("),
            Token::CloseParenthesis => String::from(")"),
            Token::OpenBracket => String::from("["),
            Token::CloseBracket => String::from("]"),
        }
    }
}
