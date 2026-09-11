use std::process::ExitCode;

use clap::Parser;
use darmoshark_cli::arguments::cli_arguments::CliArguments;
use darmoshark_cli::commands::command_runner::CommandRunner;

fn main() -> ExitCode {
  let arguments = CliArguments::parse();
  match CommandRunner::run(&arguments.command) {
    Ok(output) => {
      println!("{output}");
      ExitCode::SUCCESS
    }
    Err(error) => {
      eprintln!("error: {error}");
      ExitCode::FAILURE
    }
  }
}
