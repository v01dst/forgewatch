use crate::analysis::{create_context, ProjectHealthReport};
use crate::analyzers::run_all;
use crate::cli::{Arguments, OutputFormat};
use anyhow::Result;
mod json;
mod terminal;

pub fn run(arguments: &Arguments) -> Result<()> {
    let context = create_context(&crate::analysis::AnalysisArguments {
        path: arguments.path.clone(),
        json: arguments.output_format() == OutputFormat::Json,
        max_file_size: arguments.max_file_size * 1024,
    })?;
    let aggregate = run_all(&context);
    let report = ProjectHealthReport {
        schema_version: 1,
        project_name: aggregate.project_name,
        analyzed_path: context.root.display().to_string(),
        generated_at_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        scan: aggregate.scan,
        languages: aggregate.languages,
        dependencies: aggregate.dependencies,
        git: aggregate.repository,
        code_signals: aggregate.signals,
        structure: aggregate.structure,
        issues: aggregate.issues,
        health_score: aggregate.health_score,
        partial_analysis: aggregate.partial_analysis,
    };
    match arguments.output_format() {
        OutputFormat::Json => json::render(&report),
        OutputFormat::Terminal => terminal::render(&report, arguments.no_color, arguments.verbose),
    }
}
