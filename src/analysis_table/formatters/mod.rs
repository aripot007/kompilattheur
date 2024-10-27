mod markdown;
mod plaintext;

/// Renvoie la représentation générique d'un token, ie le nom sans les informations du token.
/// 
/// ```
/// let t = Token::Add;
/// assert_eq!(t.repr(), generic_token_repr!(t));
/// assert_eq!("<string>", generic_token_repr!(Token::String("Hello")));
/// assert_eq!("<integer>", generic_token_repr!(Token::integer(42)));
/// assert_eq!("<ident>", generic_token_repr!(Token::Identifier(IdToken {42})));
/// ```
macro_rules! generic_token_repr {
    ($token: expr) => {
        match $token {
            Token::Identifier(_) => String::from("<ident>"),
            Token::String(_) => String::from("<string>"),
            Token::Integer(_) => String::from("<integer>"),
            _ => $token.repr(),
        }
    };
}

pub (super) use generic_token_repr;

