use choreofix::{imp, shell};
#[path = "../tasks/manufacturing/mod.rs"]
mod manufacturing;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "11E" => {
            println!("{}", manufacturing::projection());
            Ok(())
        }
        [path, task] if task == "11E" => manufacturing::run_stdio(&std::fs::read(path)?).await,
        _ => Err(shell::ShellError::message(
            "usage: manufacturing-wasi GUEST.wasm 11E | --projection 11E",
        )),
    }
}
