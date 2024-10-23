pub struct NumToken {
    value: u64,
}

impl PartialEq for NumToken {
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}
impl Eq for NumToken {}


pub struct IdToken {
    pub id: usize,
}

impl PartialEq for IdToken {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}
impl Eq for IdToken {}

pub enum Token {
    Integer(NumToken),
    Identifier(IdToken),
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
    Def,
    For,
    If,
    Else,
    Return,
    Print,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Integer(num_token) => format!("<Int, {}>", num_token.value),
            Token::Identifier(id_token) => format!("<Identifier, {}>", id_token.id),
            Token::Add => String::from("<+>"),
            Token::Sub => String::from("<->"),
            Token::Mult => String::from("<*>"),
            Token::Div => String::from("<//>"),
            Token::Mod => String::from("<%>"),
            Token::Equal => String::from("<==>"),
            Token::Assign => String::from("<=>"),
            Token::NotEqual => String::from("<!=>"),
            Token::Less => String::from("< < >"),
            Token::Greater => String::from("< > >"),
            Token::LessEq => String::from("< <= >"),
            Token::GreaterEq => String::from("< >= >"),
            Token::Begin => String::from("<BEGIN>"),
            Token::End => String::from("<END>"),
            Token::Newline => String::from("<NEWLINE>"),
            Token::True => String::from("<True>"),
            Token::False => String::from("<False>"),
            Token::None => String::from("<None>"),
            Token::And => String::from("<And>"),
            Token::Or => String::from("<Or>"),
            Token::Def => String::from("<Def>"),
            Token::For => String::from("<For>"),
            Token::If => String::from("<If>"),
            Token::Else => String::from("<Else>"),
            Token::Return => String::from("<Return>"),
            Token::Print => String::from("<Print>"),
            Token::Comma => String::from("<,>"),
            Token::Sep => String::from("<:>"),
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
