use crate::lexer::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Number(f64),
    Variable(String),
    Binary {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

pub fn parse_to_expr(postfix: Vec<Token>) -> Result<Expr, anyhow::Error> {
    let mut stack: Vec<Expr> = vec![];

    for tok in postfix {
        match tok {
            Token::Number(n) => stack.push(Expr::Number(n)),
            Token::Ident(name) => stack.push(Expr::Variable(name)),
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Caret => {
                let right = stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("missing operand"))?;
                let left = stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("missing operand"))?;

                let op = match tok {
                    Token::Plus => Op::Add,
                    Token::Minus => Op::Sub,
                    Token::Star => Op::Mul,
                    Token::Slash => Op::Div,
                    Token::Caret => Op::Pow,
                    _ => return Err(anyhow::anyhow!("unknown op token: {:?}", tok)),
                };

                stack.push(Expr::Binary {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                })
            }

            _ => {
                return Err(anyhow::anyhow!("unexpected token in postfix: {:?}", tok));
            }
        }
    }

    if stack.len() != 1 {
        return Err(anyhow::anyhow!("invalid expression"));
    }

    Ok(stack.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Token,
        parser::{Expr, Op, parse_to_expr},
    };

    #[test]
    fn test_parser() -> Result<(), anyhow::Error> {
        let postfix = vec![
            Token::Ident('a'.to_string()),
            Token::Ident('b'.to_string()),
            Token::Ident('c'.to_string()),
            Token::Plus,
            Token::Star,
            Token::Ident('d'.to_string()),
            Token::Slash,
        ];

        let output = parse_to_expr(postfix)?;

        let target_output = Expr::Binary {
            op: Op::Div,
            lhs: Box::new(Expr::Binary {
                op: Op::Mul,
                lhs: Box::new(Expr::Variable("a".to_string())),
                rhs: Box::new(Expr::Binary {
                    op: Op::Add,
                    lhs: Box::new(Expr::Variable("b".to_string())),
                    rhs: Box::new(Expr::Variable("c".to_string())),
                }),
            }),
            rhs: Box::new(Expr::Variable("d".to_string())),
        };

        assert_eq!(output, target_output);

        Ok(())
    }
}
