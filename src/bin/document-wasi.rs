use choreofix::{imp, shell};

#[path = "../tasks/shared_document/mod.rs"]
mod shared_document;

#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    use choreofix::shell::ShellError;
    use hibana::runtime::resolver::DecisionArm;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (path, first) = match args.as_slice() {
        [projection, task] if projection == "--projection" && task == "7E" => {
            println!("{}", shared_document::projection());
            return Ok(());
        }
        [path, task] if task == "7E" => (path, DecisionArm::Left),
        [path, task, first] if task == "7E" && first == "WRITER_A" => (path, DecisionArm::Left),
        [path, task, first] if task == "7E" && first == "WRITER_B" => (path, DecisionArm::Right),
        _ => {
            return Err(ShellError::message(
                "usage: document-wasi GUEST.wasm 7E [WRITER_A|WRITER_B] | --projection 7E",
            ));
        }
    };
    let module = std::fs::read(path)?;
    shared_document::run_stdio(&module, first).await
}
