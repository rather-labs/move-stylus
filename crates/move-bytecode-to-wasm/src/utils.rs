#[cfg(test)]
use walrus::Module;

#[cfg(test)]
pub fn display_module(module: &mut Module) {
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    // print with line breaks
    println!("{}", wat.replace("\\n", "\n"));
}
