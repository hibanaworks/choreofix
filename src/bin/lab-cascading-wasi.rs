use choreofix::{imp, shell};
#[path = "../tasks/lab_cascading/mod.rs"]
mod lab_cascading;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| shell::ShellError::message("usage: lab-cascading-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", lab_cascading::global::programs().observation());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    lab_cascading::run_stdio(&module).await
}
