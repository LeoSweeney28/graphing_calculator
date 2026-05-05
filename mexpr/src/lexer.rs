#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,

    LParen,
    RParen,
}

impl Token {
    pub fn prec(&self) -> i32 {
        match self {
            &Self::Caret => 3,
            &Self::Star | &Self::Slash => 2,
            &Self::Plus | &Self::Minus => 1,
            _ => -1,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Self::Caret)
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, anyhow::Error> {
    let mut tokens = vec![];
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '0'..='9' => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if !(c.is_ascii_digit() || c == '.') {
                        break;
                    }
                    buf.push(c);
                    chars.next();
                }
                let n = buf
                    .parse::<f64>()
                    .map_err(|_| anyhow::anyhow!("invalid number: {}", buf))?;
                tokens.push(Token::Number(n));
            }
            'a'..='z' | 'A'..='Z' => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if !c.is_ascii_alphanumeric() {
                        break;
                    }
                    buf.push(c);
                    chars.next();
                }
                tokens.push(Token::Ident(buf));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            _ => return Err(anyhow::anyhow!("unexpected character: {}", ch)),
        }
    }

    Ok(tokens)
}

pub fn to_postfix(tokens: Vec<Token>) -> Vec<Token> {
    let mut stack: Vec<Token> = vec![];
    let mut result: Vec<Token> = vec![];

    for tok in tokens {
        match tok {
            Token::Number(_) | Token::Ident(_) => result.push(tok),
            Token::LParen => stack.push(tok),
            Token::RParen => {
                while let Some(t) = stack.pop() {
                    if t == Token::LParen {
                        break;
                    }
                    result.push(t);
                }
            }
            _ => {
                while let Some(t) = stack.last() {
                    if matches!(t, Token::LParen) {
                        break;
                    }
                    if !(t.prec() > tok.prec()
                        || (t.prec() == tok.prec() && !tok.is_right_associative()))
                    {
                        break;
                    }
                    result.push(stack.pop().unwrap());
                }
                stack.push(tok);
            }
        }
    }

    while let Some(t) = stack.pop() {
        result.push(t);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Token, lex, to_postfix};

    #[test]
    fn test_lex() -> Result<(), anyhow::Error> {
        let expr = lex("a*(b+c)/d")?;
        let target_expr = vec![
            Token::Ident('a'.to_string()),
            Token::Star,
            Token::LParen,
            Token::Ident('b'.to_string()),
            Token::Plus,
            Token::Ident('c'.to_string()),
            Token::RParen,
            Token::Slash,
            Token::Ident('d'.to_string()),
        ];

        assert_eq!(expr, target_expr);

        Ok(())
    }

    #[test]
    fn test_to_postfix() {
        // a*(b+c)/d
        let expr = vec![
            Token::Ident('a'.to_string()),
            Token::Star,
            Token::LParen,
            Token::Ident('b'.to_string()),
            Token::Plus,
            Token::Ident('c'.to_string()),
            Token::RParen,
            Token::Slash,
            Token::Ident('d'.to_string()),
        ];
        let postfix = to_postfix(expr);

        //abc+*d/
        let target_output = vec![
            Token::Ident('a'.to_string()),
            Token::Ident('b'.to_string()),
            Token::Ident('c'.to_string()),
            Token::Plus,
            Token::Star,
            Token::Ident('d'.to_string()),
            Token::Slash,
        ];
        assert_eq!(postfix, target_output);
    }
}
