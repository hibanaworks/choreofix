use choreofix::{imp, shell};
#[path = "../tasks/cascading_build/mod.rs"]
mod cascading_build;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (path, order) = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "10M" => {
            println!("{}", cascading_build::projection());
            return Ok(());
        }
        [path, task] if task == "10M" => (path, [0, 1, 2]),
        [path, task, order] if task == "10M" => {
            let order = match order.as_str() {
                "ABC" => [0, 1, 2],
                "ACB" => [0, 2, 1],
                "BAC" => [1, 0, 2],
                "BCA" => [1, 2, 0],
                "CAB" => [2, 0, 1],
                "CBA" => [2, 1, 0],
                _ => {
                    return Err(shell::ShellError::message(
                        "initial order must be a permutation of ABC",
                    ));
                }
            };
            (path, order)
        }
        _ => {
            return Err(shell::ShellError::message(
                "usage: cascading-build-wasi GUEST.wasm 10M [ABC|ACB|BAC|BCA|CAB|CBA] | --projection 10M",
            ));
        }
    };
    cascading_build::run_stdio(&std::fs::read(path)?, order).await
}
