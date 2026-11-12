use choreofix::{imp, shell};
#[path = "../tasks/multi_round_discovery/mod.rs"]
mod multi_round_discovery;
#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        shell::ShellError::message("usage: multi-round-discovery-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", multi_round_discovery::projection());
        return Ok(());
    }
    multi_round_discovery::run_stdio(&std::fs::read(path)?).await
}
