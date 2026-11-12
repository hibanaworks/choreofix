use choreofix_inputs::{
    Result, SIMULATOR_CONTRACT, restore, simulator_manifest, verify, verify_compiler,
};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (root, args) = match args.as_slice() {
        [flag, root, rest @ ..] if flag == "--root" => (PathBuf::from(root), rest),
        _ => (
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            args.as_slice(),
        ),
    };
    let result = match args {
        [mode] if mode == "restore" => restore(&root)?,
        [mode] if mode == "verify" => verify(&root)?,
        [mode] if mode == "simulator-manifest" => simulator_manifest(&root, SIMULATOR_CONTRACT)?,
        [mode] if mode == "compiler" => verify_compiler(&root)?,
        [mode] if mode == "matrix" => choreofix_inputs::matrix::write(&root)?,
        [mode, flag] if mode == "matrix" && flag == "--check" => {
            choreofix_inputs::matrix::check(&root)?
        }
        [mode, module] if mode == "wasm" => choreofix_inputs::wasm::profile(Path::new(module))?,
        [mode, module] if mode == "wash" => choreofix_inputs::wasm::wash(Path::new(module))?,
        _ => {
            return Err(
                "usage: choreofix-inputs [--root DIRECTORY] restore|verify|simulator-manifest|compiler|matrix [--check]|wasm MODULE|wash MODULE".into(),
            );
        }
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
