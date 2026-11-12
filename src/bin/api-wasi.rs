use choreofix::{imp, shell};

#[path = "../tasks/api/mod.rs"]
mod api;

#[hibana_runtime::main]
async fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: api-wasi GUEST.wasm")?;
    if path == "--projection" {
        println!("{}", api::choreofs::projection());
        return Ok(());
    }
    let module = std::fs::read(path).map_err(|e| e.to_string())?;
    api::choreofs::run_stdio(&module)
        .await
        .map_err(|e| e.to_string())
}
