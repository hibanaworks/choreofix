use choreofix::{imp, shell};

#[path = "../tasks/staged_integration/mod.rs"]
mod staged_integration;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: staged-integration-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", staged_integration::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    staged_integration::run_stdio(&module).await
}
