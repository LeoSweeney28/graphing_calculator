use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};
use wasmtime::{Engine, Func, Linker, Module as WasmtimeModule, Store, TypedFunc};

use crate::parser::{Expr, Op};

fn compile_expr(f: &mut Function, expr: &Expr) {
    match expr {
        Expr::Number(n) => f.instruction(&Instruction::F64Const((*n).into())),
        Expr::Variable(name) => {
            if name == "x" {
                f.instruction(&Instruction::LocalGet(0))
            } else if name == "y" {
                f.instruction(&Instruction::LocalGet(1))
            } else {
                f.instruction(&Instruction::F64Const(0.0.into()))
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            compile_expr(f, lhs);
            compile_expr(f, rhs);

            match op {
                Op::Add => f.instruction(&Instruction::F64Add),
                Op::Sub => f.instruction(&Instruction::F64Sub),
                Op::Mul => f.instruction(&Instruction::F64Mul),
                Op::Div => f.instruction(&Instruction::F64Div),
                Op::Pow => f.instruction(&Instruction::Call(0)),
            }
        }
    };
}

pub fn expr_to_wasm(expr: &Expr) -> anyhow::Result<Vec<u8>> {
    let mut module = Module::new();

    // Types section
    let mut types = TypeSection::new();
    types
        .ty()
        .function([ValType::F64, ValType::F64], [ValType::F64]);
    types
        .ty()
        .function([ValType::F64, ValType::F64], [ValType::F64]);
    module.section(&types);

    // Import section
    let mut imports = ImportSection::new();
    imports.import("env", "pow", wasm_encoder::EntityType::Function(1));
    module.section(&imports);

    // Functions section
    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    // Export section
    let mut exports = ExportSection::new();
    exports.export("calc", ExportKind::Func, 1);
    module.section(&exports);

    // Code section
    let mut calc_fn = Function::new([]);
    compile_expr(&mut calc_fn, expr);
    calc_fn.instruction(&Instruction::End);

    let mut code = CodeSection::new();
    code.function(&calc_fn);
    module.section(&code);

    Ok(module.finish())
}

pub fn build_func(bytes: Vec<u8>) -> anyhow::Result<(Store<()>, TypedFunc<(f64, f64), f64>)> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = WasmtimeModule::new(&engine, &bytes)?;

    let pow = Func::wrap(&mut store, |x: f64, y: f64| -> f64 { x.powf(y) });

    let mut linker = Linker::new(&engine);
    linker.define(&mut store, "env", "pow", pow)?;

    let instance = linker.instantiate(&mut store, &module)?;

    let calc = instance.get_typed_func::<(f64, f64), f64>(&mut store, "calc")?;

    Ok((store, calc))
}

#[cfg(test)]
mod tests {
    use crate::{
        jit::{build_func, compile_expr, expr_to_wasm},
        parser::{Expr, Op},
    };

    #[test]
    fn test_compiling() {
        let expr = Expr::Binary {
            op: Op::Mul,
            lhs: Box::new(Expr::Variable("x".to_string())),
            rhs: Box::new(Expr::Variable("y".to_string())),
        };
        let bytes = expr_to_wasm(&expr).unwrap();
        let (mut store, func) = build_func(bytes).unwrap();
        let result = func.call(&mut store, (5.0, 6.0)).unwrap();

        assert_eq!(result, 30.0);
    }
}
