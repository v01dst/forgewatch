use crate::analysis::{CodeSignals, StructureReport};
use crate::analysis::{HealthScore, Issue, LanguageReport, Severity};

pub fn score(
    signals: &CodeSignals,
    structure: &StructureReport,
    languages: &LanguageReport,
    issues: &[Issue],
) -> HealthScore {
    let mut score: i32 = 100;
    let sources = languages.total_source_lines;

    score -= (signals.todo.total as f64 / 100.0).ceil() as i32 * 2;
    score -= signals.long_lines.min(200) as i32 / 10;
    score -= structure.duplicate_groups.len().min(10) as i32 * 3;

    if let Some(untested) = structure.source_files_without_test_coverage_estimate {
        if sources > 0 {
            let penalty = ((untested as f64 / sources as f64).min(1.0) * 15.0) as i32;
            score -= penalty;
        }
    }

    for issue in issues {
        match issue.severity {
            Severity::Info => score -= 1,
            Severity::Warning => score -= 6,
            Severity::Critical => score -= 18,
        }
    }

    let score = score.clamp(0, 100) as u8;
    let grade = if score >= 90 {
        "A"
    } else if score >= 75 {
        "B"
    } else if score >= 60 {
        "C"
    } else if score >= 40 {
        "D"
    } else {
        "F"
    };
    HealthScore {
        score,
        grade: grade.to_string(),
    }
}
