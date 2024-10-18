pub struct SimpleToken {
    // tag est un identifiant unique pour chaque token
    pub tag: usize,
}

pub struct IntToken {
    pub tag: usize,
    // stocke la valeur de l'entier
    pub value: usize,
}

pub struct IdToken {
    pub tag: usize,
    // stocke le nom de l'identifiant
    pub name: String,
}

pub struct KeywordToken {
    pub tag: usize,
    // stocke le mot clé
    pub name: String,
}

pub enum Token {
    Simple(SimpleToken),
    Integer(IntToken),
    Identifier(IdToken),
    Keyword(KeywordToken),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Simple(a), Token::Simple(b)) => a.tag == b.tag,
            (Token::Integer(a), Token::Integer(b)) => a.tag == b.tag,
            (Token::Identifier(a), Token::Identifier(b)) => a.tag == b.tag,
            (Token::Keyword(a), Token::Keyword(b)) => a.tag == b.tag,
            _ => false,
        }
    }
}

impl Token {
    pub fn simple(tag: usize) -> Token {
        Token::Simple(SimpleToken { tag })
    }
    pub fn integer(tag: usize, value: usize) -> Token {
        Token::Integer(IntToken { tag, value })
    }
    pub fn identifier(tag: usize, name: String) -> Token {
        Token::Identifier(IdToken { tag, name })
    }
    pub fn keyword(tag: usize, name: String) -> Token {
        Token::Keyword(KeywordToken { tag, name })
    }
    pub fn get_tag(&self) -> usize {
        match self {
            Token::Simple(token) => token.tag,
            Token::Integer(token) => token.tag,
            Token::Identifier(token) => token.tag,
            Token::Keyword(token) => token.tag,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Token::Simple(token) => format!("<Simple,{}>", token.tag),
            Token::Integer(token) => format!("<Int,{}>", token.value),
            Token::Identifier(token) => format!("<Id,{}>", token.name),
            Token::Keyword(token) => format!("<Keyword,{}>", token.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_token() {
        let token = SimpleToken { tag: 1 };
        assert_eq!(token.tag, 1);
    }

    #[test]
    fn test_int_token() {
        let token = IntToken { tag: 1, value: 10 };
        assert_eq!(token.tag, 1);
        assert_eq!(token.value, 10);
    }

    #[test]
    fn test_id_token() {
        let token = IdToken {
            tag: 1,
            name: "test".to_string(),
        };
        assert_eq!(token.tag, 1);
        assert_eq!(token.name, "test");
    }

    #[test]
    fn test_keyword_token() {
        let token = KeywordToken {
            tag: 1,
            name: "test".to_string(),
        };
        assert_eq!(token.tag, 1);
        assert_eq!(token.name, "test");
    }

    #[test]
    fn test_token_enum() {
        let simple_token = Token::simple(1);
        let int_token = Token::integer(1, 10);
        let id_token = Token::identifier(1, "test".to_string());
        let keyword_token = Token::keyword(1, "test".to_string());

        if Token::simple(1) == simple_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::integer(1, 10) == int_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::identifier(1, "test".to_string()) == id_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::keyword(1, "test".to_string()) == keyword_token {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_token_neq() {
        let simple_token = Token::simple(1);
        let int_token = Token::integer(1, 10);
        let id_token = Token::identifier(1, "test".to_string());
        let keyword_token = Token::keyword(1, "test".to_string());

        if Token::simple(2) != simple_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::integer(2, 10) != int_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::identifier(2, "test".to_string()) != id_token {
            assert!(true);
        } else {
            assert!(false);
        }

        if Token::keyword(2, "test".to_string()) != keyword_token {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_token_get_tag() {
        let simple_token = Token::simple(1);
        let int_token = Token::integer(1, 10);
        let id_token = Token::identifier(1, "test".to_string());
        let keyword_token = Token::keyword(1, "test".to_string());

        assert_eq!(simple_token.get_tag(), 1);
        assert_eq!(int_token.get_tag(), 1);
        assert_eq!(id_token.get_tag(), 1);
        assert_eq!(keyword_token.get_tag(), 1);
    }
}
