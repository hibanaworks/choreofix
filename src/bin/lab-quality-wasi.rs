use choreofix::{imp, shell};

#[path = "../tasks/lab_quality/mod.rs"]
mod lab_quality;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: lab-quality-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", lab_quality::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    lab_quality::run_stdio(&module).await
}
