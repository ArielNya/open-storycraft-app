//! Open Storycraft CLI entry point.

#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use std::process::ExitCode;

fn main() -> ExitCode {
    storycraft_cli::init_tracing();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(storycraft_cli::run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => storycraft_cli::exit_err(&err),
    }
}
