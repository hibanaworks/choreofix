use choreofix::{imp, shell};

#[path = "../tasks/shared_document/mod.rs"]
mod shared_document;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    use choreofix::shell::ShellError;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let path = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "7E" => {
            println!("{}", shared_document::projection());
            return Ok(());
        }
        [path, task] if task == "7E" => path,
        _ => {
            return Err(ShellError::message(
                "usage: document-wasi GUEST.wasm 7E | --projection 7E",
            ));
        }
    };
    let module = std::fs::read(path)?;
    shared_document::run_stdio(&module).await
}
