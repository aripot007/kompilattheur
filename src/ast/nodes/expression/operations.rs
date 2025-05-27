use std::fmt::Display;

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
    FLOATADD,
    FLOATSUB,
    FLOATMULT,
    FLOATDIV,
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
            Token::FloatAdd => BinOp::FLOATADD,
            Token::FloatSub => BinOp::FLOATSUB,
            Token::FloatMult => BinOp::FLOATMULT,
            Token::FloatDiv => BinOp::FLOATDIV,
            Token::Equal => BinOp::EQ,
            Token::NotEqual => BinOp::NEQ,
            Token::Less => BinOp::LESS,
            Token::Greater => BinOp::GREATER,
            Token::LessEq => BinOp::LESSEQ,
            Token::GreaterEq => BinOp::GREATEREQ,
            _ => panic!("Cannot convert {} token to operation", value),
        }
    }
}

impl Into<&'static str> for BinOp {
    fn into(self) -> &'static str {
        match self {
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
            BinOp::FLOATADD => ".+",
            BinOp::FLOATSUB => ".-",
            BinOp::FLOATMULT => ".*",
            BinOp::FLOATDIV => "./",
            BinOp::ACCESS => "ACCESS",
            BinOp::AND => "AND",
            BinOp::OR => "OR",
        }
    }
}

impl Into<String> for BinOp {
    fn into(self) -> String {
        let s: &str = self.into();
        String::from(s)
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(*self))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    NOT,
    NEG,
    FLOATNEG,
}

impl From<Token> for UnOp {
    fn from(value: Token) -> Self {
        match value {
            Token::Not => UnOp::NOT,
            Token::Sub => UnOp::NEG,
            Token::FloatSub => UnOp::FLOATNEG,
            _ => panic!("Cannot convert token {} to unary operator", value),
        }
    }
}

impl Into<&'static str> for UnOp {
    fn into(self) -> &'static str {
        match self {
            UnOp::NOT => "NOT",
            UnOp::NEG => "NEG",
            UnOp::FLOATNEG => "FNEG",
        }
    }
}

impl Into<String> for UnOp {
    fn into(self) -> String {
        let s: &str = self.into();
        String::from(s)
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(*self))
    }
}
