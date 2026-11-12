use choreofix::{imp, shell};

#[path = "../tasks/codebase/mod.rs"]
mod codebase;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: codebase-wasi GUEST.wasm 1E|1M|1H")
    })?;
    let level = args
        .next()
        .ok_or_else(|| choreofix::shell::ShellError::message("missing codebase task"))?;
    if path == "--projection" {
        println!("{}", codebase::choreofs::projection(&level)?);
        return Ok(());
    }
    let module = std::fs::read(path)?;
    codebase::choreofs::run_stdio(&module, &level).await
}
