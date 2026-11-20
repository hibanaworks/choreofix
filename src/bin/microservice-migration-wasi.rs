use choreofix::{imp, shell};

#[path = "../tasks/microservice_migration/mod.rs"]
mod microservice_migration;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        shell::ShellError::message("usage: microservice-migration-wasi GUEST.wasm")
    })?;
    if path == "--projection" {
        println!("{}", microservice_migration::Programs::new().observation());
        return Ok(());
    }
    microservice_migration::run_stdio(&std::fs::read(path)?).await
}
