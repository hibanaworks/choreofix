use choreofix::{imp, shell};

#[path = "../tasks/critical_care/mod.rs"]
mod critical_care;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: critical-care-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", critical_care::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    critical_care::run_stdio(&module).await
}
