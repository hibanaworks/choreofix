use choreofix::{imp, shell};
#[path = "../tasks/regulatory_discovery/mod.rs"]
mod regulatory_discovery;
#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: regulatory-discovery-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", regulatory_discovery::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    regulatory_discovery::run_stdio(&module).await
}
