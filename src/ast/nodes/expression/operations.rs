use crate::common::types::Token;

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    AND,
    OR,
    LESS,
    LESSEQ,
    GREATER,
    GREATEREQ,
    EQ,
    NEQ,
    MULT,
    DIV,
    MOD,
    ADD,
    SUB,
    ACCESS,
}

impl From<Token> for BinOp {
    fn from(value: Token) -> Self {
        match value {
            Token::Or => BinOp::OR,
            Token::And => BinOp::AND,
            Token::Add => BinOp::ADD,
            Token::Sub => BinOp::SUB,
            Token::Mult => BinOp::MULT,
            Token::Div => BinOp::DIV,
            Token::Mod => BinOp::MOD,
            Token::Equal => BinOp::EQ,
            Token::NotEqual => BinOp::NEQ,
            Token::Less => BinOp::LESS,
            Token::Greater => BinOp::GREATER,
            Token::LessEq => BinOp::LESSEQ,
            Token::GreaterEq => BinOp::GREATEREQ,
            _ => panic!("Cannot convert {} token to operation", value)
        }
    }
}

impl Into<String> for BinOp {
    fn into(self) -> String {
        String::from(match self {
            BinOp::LESS => "<",
            BinOp::LESSEQ => "<=",
            BinOp::GREATER => ">",
            BinOp::GREATEREQ => ">=",
            BinOp::EQ => "==",
            BinOp::NEQ => "!=",
            BinOp::MULT => "*",
            BinOp::DIV => "//",
            BinOp::MOD => "%",
            BinOp::ADD => "+",
            BinOp::SUB => "-",
            BinOp::ACCESS => "ACCESS",
            BinOp::AND => "AND",
            BinOp::OR => "OR",
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    NOT,
    NEG,
}

impl From<Token> for UnOp {
    fn from(value: Token) -> Self {
        match value {
            Token::Not => UnOp::NOT,
            Token::Sub => UnOp::NEG,
            _ => panic!("Cannot convert token {} to unary operator", value),
        }
    }
}

impl Into<String> for UnOp {
    fn into(self) -> String {
        String::from(match self {
            UnOp::NOT => "NOT",
            UnOp::NEG => "NEG",
        })
    }
}
