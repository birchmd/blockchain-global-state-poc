extern crate clap;
extern crate parity_wasm;
extern crate pwasm_utils;

use clap::{App, Arg};
use parity_wasm::elements::{deserialize_file, serialize_to_file, Module};
use pwasm_utils::externalize_mem;
use std::path::PathBuf;

fn main() {
    let matches = App::new("wasm_prep")
        .arg(
            Arg::with_name("target")
                .index(1)
                .required(true)
                .help("Cargo target directory"),
        )
        .arg(
            Arg::with_name("wasm")
                .index(2)
                .required(true)
                .help("Wasm binary name"),
        )
        .arg(
            Arg::with_name("final_name")
                .help("Final wasm binary name")
                .takes_value(true)
                .long("final"),
        )
        .get_matches();

    let target_dir = matches.value_of("target").unwrap();
    let wasm_binary = matches.value_of("wasm").unwrap();
    let final_binary = matches.value_of("final_name").unwrap_or(wasm_binary);

    let mut wasm_path: PathBuf = [target_dir, "wasm32-unknown-unknown", "release", wasm_binary]
        .iter()
        .collect();
    let mut wasm_out_path: PathBuf = [target_dir, final_binary].iter().collect();
    wasm_path.set_extension("wasm");
    wasm_out_path.set_extension("wasm");

    let module = deserialize_file(wasm_path).expect("Failed to load wasm module");
    let final_module = process(module);

    serialize_to_file(wasm_out_path, final_module).expect("Failed to write to output");
}

fn process(module: Module) -> Module {
    let mut ext_mod = externalize_mem(module, None, 128);
    remove_memory_export(&mut ext_mod);
    ext_mod
}

fn remove_memory_export(module: &mut Module) {
    let exports = module
        .export_section_mut()
        .expect("Exports section must exist");
    let entries = exports.entries_mut();
    let mem_pos = entries.iter().position(|e| e.field() == "memory");
    if let Some(mem_pos) = mem_pos {
        entries.remove(mem_pos);
    }
}
