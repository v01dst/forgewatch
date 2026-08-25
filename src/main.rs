use clap::Parser;
use forgewatch::cli::Arguments;
use forgewatch::report;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = Arguments::parse();
    match report::run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if arguments.json {
                let payload = serde_json::json!({
                    "error": {"message": error.to_string()}
                });
                eprintln!("{payload}");
            } else {
                eprintln!("forgewatch: {error}");
            }
            ExitCode::FAILURE
        }
    }
}
