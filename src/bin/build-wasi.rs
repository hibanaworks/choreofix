use choreofix::{imp, shell};
#[path = "../tasks/shared_build/mod.rs"]
mod shared_build;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let path = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "10E" => {
            println!("{}", shared_build::projection());
            return Ok(());
        }
        [path, task] if task == "10E" => path,
        _ => {
            return Err(shell::ShellError::message(
                "usage: build-wasi GUEST.wasm 10E | --projection 10E",
            ));
        }
    };
    shared_build::run_stdio(&std::fs::read(path)?).await
}
