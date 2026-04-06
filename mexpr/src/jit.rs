use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    MemArg, MemorySection, MemoryType, Module, TypeSection, ValType,
};
use wasmtime::{Engine, Func, Linker, Memory, Module as WasmtimeModule, Store, TypedFunc};

use crate::parser::{Expr, Op};

fn unroll_constant_exponent(f: &mut Function, base: &Expr, exp: u32) {
    if exp == 0 {
        f.instruction(&Instruction::F64Const(1.0.into()));
        return;
    }

    let local_base = 2;
    compile_expr(f, base);
    f.instruction(&Instruction::LocalSet(local_base));

    f.instruction(&Instruction::F64Const(1.0.into()));

    let mut n = exp;
    while n > 0 {
        if n % 2 == 1 {
            f.instruction(&Instruction::LocalGet(local_base));
            f.instruction(&Instruction::F64Mul);
        }

        n /= 2;
        if n > 0 {
            f.instruction(&Instruction::LocalGet(local_base));
            f.instruction(&Instruction::LocalGet(local_base));
            f.instruction(&Instruction::F64Mul);
            f.instruction(&Instruction::LocalSet(local_base));
        }
    }
}

fn compile_expr(f: &mut Function, expr: &Expr) {
    match expr {
        Expr::Number(n) => {
            f.instruction(&Instruction::F64Const((*n).into()));
        }
        Expr::Variable(name) => {
            if name == "x" {
                f.instruction(&Instruction::LocalGet(0));
            } else if name == "y" {
                f.instruction(&Instruction::LocalGet(1));
            } else {
                f.instruction(&Instruction::F64Const(0.0.into()));
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            if !matches!(op, Op::Pow) {
                compile_expr(f, lhs);
                compile_expr(f, rhs);
            }

            match op {
                Op::Add => {
                    f.instruction(&Instruction::F64Add);
                }
                Op::Sub => {
                    f.instruction(&Instruction::F64Sub);
                }
                Op::Mul => {
                    f.instruction(&Instruction::F64Mul);
                }
                Op::Div => {
                    f.instruction(&Instruction::F64Div);
                }
                Op::Pow => {
                    const MAX_INT_EXPONENT: i32 = 1_000_000;
                    if let Expr::Number(n) = **rhs {
                        if n.fract() == 0.0 && (n as i32).abs() < MAX_INT_EXPONENT {
                            let exp = n as i32;
                            if exp == 0 {
                                f.instruction(&Instruction::F64Const(1.0.into()));
                            } else if exp > 0 {
                                unroll_constant_exponent(f, lhs, exp as u32);
                            } else {
                                f.instruction(&Instruction::F64Const(1.0.into()));
                                unroll_constant_exponent(f, lhs, (-exp) as u32);
                                f.instruction(&Instruction::F64Div);
                            }
                            return;
                        }
                    }

                    compile_expr(f, lhs);
                    compile_expr(f, rhs);
                    // pow(lhs, rhs)
                    f.instruction(&Instruction::Call(0));
                }
            }
        }
    };
}

fn generate_calc_all(f: &mut Function) {
    // ptr_out = ptr_in + len * 16
    f.instruction(&Instruction::LocalGet(0)); // ptr_in
    f.instruction(&Instruction::LocalGet(1)); // len
    f.instruction(&Instruction::I32Const(16));
    f.instruction(&Instruction::I32Mul);
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::LocalSet(3)); // ptr_out

    // i = 0
    f.instruction(&Instruction::I32Const(0));
    f.instruction(&Instruction::LocalSet(2));

    // block + loop
    f.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
    f.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

    // if i >= len -> break
    f.instruction(&Instruction::LocalGet(2));
    f.instruction(&Instruction::LocalGet(1));
    f.instruction(&Instruction::I32GeU);
    f.instruction(&Instruction::BrIf(1));

    // load x (ptr_in + i * 16)
    f.instruction(&Instruction::LocalGet(0)); // ptr_in
    f.instruction(&Instruction::LocalGet(2)); // i
    f.instruction(&Instruction::I32Const(16));
    f.instruction(&Instruction::I32Mul);
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::F64Load(MemArg {
        offset: 0,
        align: 3, // 2^3 = 8 bytes for f64
        memory_index: 0,
    }));

    // load y (ptr_in + i * 16 + 8)
    f.instruction(&Instruction::LocalGet(0)); // ptr_in
    f.instruction(&Instruction::LocalGet(2)); // i
    f.instruction(&Instruction::I32Const(16));
    f.instruction(&Instruction::I32Mul);
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::I32Const(8));
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::F64Load(MemArg {
        offset: 0,
        align: 3, // 2^3 = 8 bytes for f64
        memory_index: 0,
    }));

    f.instruction(&Instruction::Call(1)); // calc(x, y)

    // Preserve computed value while calculating the destination address.
    f.instruction(&Instruction::LocalSet(4));

    // store result (ptr_out + i * 8)
    f.instruction(&Instruction::LocalGet(3)); // ptr_out
    f.instruction(&Instruction::LocalGet(2)); // i
    f.instruction(&Instruction::I32Const(8));
    f.instruction(&Instruction::I32Mul);
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::LocalGet(4)); // value

    f.instruction(&Instruction::F64Store(MemArg {
        offset: 0,
        align: 3, // 2^3 = 8 bytes for f64
        memory_index: 0,
    }));

    // i++
    f.instruction(&Instruction::LocalGet(2));
    f.instruction(&Instruction::I32Const(1));
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::LocalSet(2));

    // loop
    f.instruction(&Instruction::Br(0));

    f.instruction(&Instruction::End); // loop
    f.instruction(&Instruction::End); // block

    // return ptr_out
    f.instruction(&Instruction::LocalGet(3));
    f.instruction(&Instruction::End);
}

pub fn expr_to_wasm(expr: &Expr) -> anyhow::Result<Vec<u8>> {
    let mut module = Module::new();

    // Types section
    let mut types = TypeSection::new();
    types
        .ty()
        .function([ValType::F64, ValType::F64], [ValType::F64]); // calc function
    types
        .ty()
        .function([ValType::F64, ValType::F64], [ValType::F64]); // pow function
    types
        .ty()
        .function([ValType::I32, ValType::I32], [ValType::I32]); // calc_all function
    module.section(&types);

    // Import section
    let mut imports = ImportSection::new();
    imports.import("env", "pow", wasm_encoder::EntityType::Function(1));
    module.section(&imports);

    // Functions section
    let mut functions = FunctionSection::new();
    functions.function(0);
    functions.function(2);
    module.section(&functions);

    // Memory section
    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memory);

    // Export section
    let mut exports = ExportSection::new();
    exports.export("calc", ExportKind::Func, 1);
    exports.export("calc_all", ExportKind::Func, 2);

    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    // Code section
    let mut calc_fn = Function::new([(1, ValType::F64)]);
    compile_expr(&mut calc_fn, expr);
    calc_fn.instruction(&Instruction::End);

    let mut calc_all_fn = Function::new([
        (2, ValType::I32), // i, ptr_out
        (1, ValType::F64), // temporary result value
    ]);
    generate_calc_all(&mut calc_all_fn);

    let mut code = CodeSection::new();
    code.function(&calc_fn);
    code.function(&calc_all_fn);
    module.section(&code);

    Ok(module.finish())
}

pub fn build_func(
    bytes: Vec<u8>,
) -> anyhow::Result<(
    Store<()>,
    TypedFunc<(f64, f64), f64>,
    TypedFunc<(i32, i32), i32>,
    Memory,
)> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = WasmtimeModule::new(&engine, &bytes)?;

    let pow = Func::wrap(&mut store, |x: f64, y: f64| -> f64 { x.powf(y) });

    let mut linker = Linker::new(&engine);
    linker.define(&mut store, "env", "pow", pow)?;

    let instance = linker.instantiate(&mut store, &module)?;

    let calc = instance.get_typed_func::<(f64, f64), f64>(&mut store, "calc")?;

    let calc_all = instance.get_typed_func::<(i32, i32), i32>(&mut store, "calc_all")?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("memory export not found");

    Ok((store, calc, calc_all, memory))
}

#[cfg(test)]
mod tests {
    use crate::{
        jit::{build_func, expr_to_wasm},
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
        let (mut store, func, _batch_func, _memory) = build_func(bytes).unwrap();
        let result = func.call(&mut store, (5.0, 6.0)).unwrap();

        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_batch_compiling() {
        let expr = Expr::Binary {
            op: Op::Add,
            lhs: Box::new(Expr::Variable("x".to_string())),
            rhs: Box::new(Expr::Variable("y".to_string())),
        };

        let bytes = expr_to_wasm(&expr).unwrap();
        let (mut store, _func, batch_func, memory) = build_func(bytes).unwrap();

        let inputs: [(f64, f64); 3] = [(1.0, 2.0), (3.5, 4.5), (-1.0, 0.25)];
        let len = inputs.len() as i32;
        let input_bytes = inputs.len() * 16;
        let output_bytes = inputs.len() * 8;
        assert!(memory.data_size(&store) >= input_bytes + output_bytes);

        {
            let data = memory.data_mut(&mut store);
            for (i, (x, y)) in inputs.iter().enumerate() {
                let base = i * 16;
                data[base..base + 8].copy_from_slice(&x.to_le_bytes());
                data[base + 8..base + 16].copy_from_slice(&y.to_le_bytes());
            }
        }

        let out_ptr = batch_func.call(&mut store, (0, len)).unwrap();
        assert_eq!(out_ptr as usize, input_bytes);

        let data = memory.data(&store);
        for (i, (x, y)) in inputs.iter().enumerate() {
            let base = out_ptr as usize + i * 8;
            let mut buf = [0_u8; 8];
            buf.copy_from_slice(&data[base..base + 8]);
            let result = f64::from_le_bytes(buf);
            assert!((result - (x + y)).abs() < 1e-12);
        }
    }
}
