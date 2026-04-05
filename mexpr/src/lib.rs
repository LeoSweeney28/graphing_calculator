pub use wasmtime::{Store, TypedFunc};

use crate::{
    jit::{build_func, expr_to_wasm},
    lexer::{lex, to_postfix},
    parser::parse_to_expr,
};

mod jit;
mod lexer;
mod parser;

pub fn compile_function(input: &str) -> anyhow::Result<(Store<()>, TypedFunc<(f64, f64), f64>)> {
    let tokens = lex(input)?;
    let postfix = to_postfix(tokens);
    let expr = parse_to_expr(postfix)?;
    let bytes = expr_to_wasm(&expr)?;
    build_func(bytes)
}

#[cfg(test)]
mod tests {
    use crate::compile_function;

    #[test]
    fn test_combined() -> anyhow::Result<()> {
        let input = "((5.0 * 3 - 1) ^ 2)+1 - x*y";
        let (mut store, func) = compile_function(input)?;

        let result = func.call(&mut store, (6.0, 8.0))?;

        assert_eq!(result, 149.0);
        Ok(())
    }
}
