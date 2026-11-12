use choreofix::{imp, shell};

#[path = "../tasks/multi_specialty/mod.rs"]
mod multi_specialty;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: multi-specialty-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", multi_specialty::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    multi_specialty::run_stdio(&module).await
}
