use choreofix::{imp, shell};

#[path = "../tasks/survey_paper/mod.rs"]
mod survey_paper;

#[hibana_runtime::main]
async fn main() -> shell::ShellResult<()> {
    let argument = std::env::args().nth(1).ok_or_else(|| {
        shell::ShellError::message("usage: survey-paper-wasi GUEST.wasm | --projection")
    })?;
    if argument == "--projection" {
        return survey_paper::runtime::run(None).await;
    }
    let module = std::fs::read(argument)?;
    survey_paper::runtime::run(Some(&module)).await
}
