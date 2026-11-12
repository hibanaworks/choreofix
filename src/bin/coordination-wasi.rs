use choreofix::{imp, shell};

#[path = "../tasks/api_coordination/mod.rs"]
mod api_coordination;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: coordination-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", api_coordination::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    api_coordination::run_stdio(&module).await
}
