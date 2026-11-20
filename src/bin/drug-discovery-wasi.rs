use choreofix::{imp, shell};
#[path = "../tasks/drug_discovery/mod.rs"]
mod drug_discovery;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: drug-discovery-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", drug_discovery::global::programs().observation());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    drug_discovery::run_stdio(&module).await
}
