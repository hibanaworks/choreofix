use choreofix::{imp, shell};

#[path = "../tasks/research_report/mod.rs"]
mod research_report;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: report-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", research_report::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    research_report::run_stdio(&module).await
}
