use choreofix::{imp, shell};

#[path = "../tasks/research_paper/mod.rs"]
mod research_paper;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: research-paper-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", research_paper::global::programs().observation());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    research_paper::run_stdio(&module).await
}
