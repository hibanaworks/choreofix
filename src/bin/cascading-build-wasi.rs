use choreofix::{imp, shell};
#[path = "../tasks/cascading_build/mod.rs"]
mod cascading_build;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let path = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "10M" => {
            println!("{}", cascading_build::projection());
            return Ok(());
        }
        [path, task] if task == "10M" => path,
        _ => {
            return Err(shell::ShellError::message(
                "usage: cascading-build-wasi GUEST.wasm 10M | --projection 10M",
            ));
        }
    };
    cascading_build::run_stdio(&std::fs::read(path)?).await
}
