use choreofix::{imp, shell};

#[path = "../tasks/lab_synthesis/mod.rs"]
mod lab_synthesis;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: lab-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", lab_synthesis::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    lab_synthesis::run_stdio(&module).await
}
