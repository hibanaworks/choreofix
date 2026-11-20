use choreofix::{imp, shell};

#[path = "../tasks/shared_document/mod.rs"]
mod shared_document;

#[cfg(not(test))]
#[path = "../tasks/figure_document/mod.rs"]
mod figure_document;

#[cfg(not(test))]
#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    use choreofix::shell::ShellError;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (path, task) = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "7E" => {
            println!("{}", shared_document::projection());
            return Ok(());
        }
        [projection, task] if projection == "--projection" && task == "7M" => {
            println!("{}", figure_document::projection());
            return Ok(());
        }
        [path, task] if matches!(task.as_str(), "7E" | "7M") => (path, task),
        _ => {
            return Err(ShellError::message(
                "usage: document-wasi GUEST.wasm 7E|7M or --projection 7E|7M",
            ));
        }
    };
    let module = std::fs::read(path)?;
    if task == "7M" {
        figure_document::run_stdio(&module).await
    } else {
        shared_document::run_stdio(&module).await
    }
}
