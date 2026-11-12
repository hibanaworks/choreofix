use choreofix::{imp, shell};

#[path = "../tasks/cicd/mod.rs"]
mod cicd;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: cicd-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", cicd::choreofs::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    cicd::choreofs::run_stdio(&module).await
}
