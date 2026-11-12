use choreofix::{imp, shell};
#[path = "../tasks/kitchen/mod.rs"]
mod kitchen;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [flag, task] if flag == "--projection" && task == "12E" => {
            println!("{}", kitchen::projection());
            Ok(())
        }
        [path, task] if task == "12E" => kitchen::run_stdio(&std::fs::read(path)?).await,
        _ => Err(shell::ShellError::message(
            "usage: kitchen-wasi GUEST.wasm 12E | --projection 12E",
        )),
    }
}
