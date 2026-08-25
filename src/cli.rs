use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Terminal,
    Json,
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "forgewatch",
    version,
    about = "Analyze the technical health of a local software project",
    after_help = "Examples:\n  forgewatch .\n  forgewatch /path/to/project\n  forgewatch --json ."
)]
pub struct Arguments {
    /// Project directory to analyze
    pub path: PathBuf,

    /// Output machine-readable JSON using the documented stable schema
    #[arg(long)]
    pub json: bool,

    /// Select the presentation layer explicitly
    #[arg(long, value_enum, default_value_t = OutputFormat::Terminal)]
    pub format: OutputFormat,

    /// Do not colorize terminal output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Skip files larger than this size in KiB during content inspection
    #[arg(long = "max-file-size", default_value_t = 8_192)]
    pub max_file_size: u64,

    /// Show detailed evidence in terminal mode
    #[arg(short, long)]
    pub verbose: bool,
}

impl Arguments {
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            self.format
        }
    }
}
