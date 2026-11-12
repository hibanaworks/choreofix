use choreofix::{imp, shell};

#[path = "../tasks/shared_refactor/mod.rs"]
mod shared_refactor;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: refactor-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", shared_refactor::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    shared_refactor::run_stdio(&module).await
}
