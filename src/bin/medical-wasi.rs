use choreofix::{imp, shell};

#[path = "../tasks/medical_consultation/mod.rs"]
mod medical_consultation;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: medical-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", medical_consultation::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    medical_consultation::run_stdio(&module).await
}
