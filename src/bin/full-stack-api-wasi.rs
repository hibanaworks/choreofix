use choreofix::{imp, shell};

#[path = "../tasks/full_stack_api/mod.rs"]
mod full_stack_api;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: full-stack-api-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", full_stack_api::programs().observation());
        return Ok(());
    }
    let wash = std::fs::read(path)?;
    full_stack_api::run_stdio(&wash).await
}
