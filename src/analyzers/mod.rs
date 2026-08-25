pub mod dependencies;
pub mod git;
pub mod issues;
pub mod languages;
pub mod metrics;
pub mod score;
pub mod structure;

use crate::analysis::{AggregateReport, AnalysisContext};

pub fn run_all(context: &AnalysisContext) -> AggregateReport {
    let languages = languages::analyze(context);
    let dependencies = dependencies::analyze(context);
    let repository = git::analyze(&context.root);
    let signals = metrics::analyze(context, &languages.primary);
    let structure = structure::analyze(context, &languages);
    let issues = issues::analyze(context, &signals, &structure, &repository);
    let health_score = score::score(&signals, &structure, &languages, &issues);

    AggregateReport {
        project_name: context.project_name(),
        scan: crate::analysis::ScanSummary {
            files_scanned: context.input.files.len() as u64,
        },
        languages,
        dependencies,
        repository,
        signals,
        structure,
        issues,
        health_score,
        partial_analysis: context.partial_analysis.clone(),
    }
}
