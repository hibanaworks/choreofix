use choreofix::{imp, shell};

#[path = "../tasks/cicd_pipeline/mod.rs"]
mod pipeline;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: cicd-pipeline-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", pipeline::projection());
        return Ok(());
    }
    let program = std::fs::read(path)?;
    pipeline::run_stdio(&program).await
}
