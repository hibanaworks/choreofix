use choreofix::{imp, shell};

#[path = "../tasks/large_document/mod.rs"]
mod large_document;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: large-document-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", large_document::Programs::new().observation());
        return Ok(());
    }
    large_document::run_stdio(&std::fs::read(path)?).await
}
