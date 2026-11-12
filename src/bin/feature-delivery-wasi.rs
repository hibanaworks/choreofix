use choreofix::{imp, shell};

#[path = "../tasks/feature_delivery/mod.rs"]
mod feature_delivery;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: feature-delivery-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", feature_delivery::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    feature_delivery::run_stdio(&module).await
}
