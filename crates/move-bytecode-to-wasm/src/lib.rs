use std::path::Path;

use move_package::compilation::compiled_package::CompiledPackage;
use walrus::{FunctionBuilder, Module, ModuleConfig, ValType, ir::BinaryOp};

pub fn translate_package(package: &CompiledPackage, rerooted_path: &Path) {
    // Construct a new Walrus module.
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);
    let memory_id = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory_id);

    let pay_for_memory_grow_type = module.types.add(&[ValType::I32], &[]);
    let (pay_for_memory_grow, _) =
        module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type);

    // Building this factorial implementation:
    // https://github.com/WebAssembly/testsuite/blob/7816043/fac.wast#L46-L66
    let mut factorial = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    // Create our parameter and our two locals.
    let n = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let res = module.locals.add(ValType::I32);

    factorial
        // Enter the function's body.
        .func_body()
        // (local.set $i (local.get $n))
        .local_get(n)
        .local_set(i)
        // (local.set $res (i32.const 1))
        .i32_const(1)
        .local_set(res)
        .block(None, |done| {
            let done_id = done.id();
            done.loop_(None, |loop_| {
                let loop_id = loop_.id();
                loop_
                    // (i32.eq (local.get $i) (i32.const 0))
                    .local_get(i)
                    .i32_const(0)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then| {
                            // (then (br $done))
                            then.br(done_id);
                        },
                        |else_| {
                            else_
                                // (local.set $res (i32.mul (local.get $i) (local.get $res)))
                                .local_get(i)
                                .local_get(res)
                                .binop(BinaryOp::I32Mul)
                                .local_set(res)
                                // (local.set $i (i32.sub (local.get $i) (i32.const 1))))
                                .local_get(i)
                                .i32_const(1)
                                .binop(BinaryOp::I32Sub)
                                .local_set(i);
                        },
                    )
                    .br(loop_id);
            });
        })
        .local_get(res);

    let factorial = factorial.finish(vec![n], &mut module.funcs);

    // Export the `factorial` function.
    module.exports.add("user_entrypoint", factorial);

    // Emit the `.wasm` binary to the `target/out.wasm` file.
    println!("WASM MODULE: {:#?}", module.debug);
    module
        .emit_wasm_file(rerooted_path.join("out.wasm"))
        .unwrap();

    // Convert to WAT format
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    std::fs::write(rerooted_path.join("output.wat"), wat.as_bytes())
        .expect("Failed to write WAT file");
    println!("WAT file generated: output.wat");
}
