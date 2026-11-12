use choreofix::{imp, shell};

#[path = "../tasks/review/mod.rs"]
mod review;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: review-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", review::choreofs::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    review::choreofs::run_stdio(&module).await
}
