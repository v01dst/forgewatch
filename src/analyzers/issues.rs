use crate::analysis::{
    AnalysisContext, CodeSignals, Issue, RepositoryReport, Severity, StructureReport,
};

pub fn analyze(
    context: &AnalysisContext,
    signals: &CodeSignals,
    structure: &StructureReport,
    repository: &RepositoryReport,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let source_count = context.input.files.len() as u64;

    if source_count == 0 {
        issues.push(Issue {
            severity: Severity::Critical,
            category: "structure".into(),
            title: "No readable source files".into(),
            detail: "The project scan found no analyzable source files.".into(),
            evidence: vec![],
        });
    } else if !structure.has_tests && source_count >= 5 {
        issues.push(Issue {
            severity: Severity::Warning,
            category: "testing".into(),
            title: "No tests detected".into(),
            detail: "No conventional test directories or test filenames were found.".into(),
            evidence: vec![],
        });
    }

    if !signals.large_files.is_empty() {
        issues.push(Issue {
            severity: Severity::Warning,
            category: "maintainability".into(),
            title: format!(
                "Large source files detected ({})",
                signals.large_files.len()
            ),
            detail: "Large files can make review and maintenance harder.".into(),
            evidence: signals
                .large_files
                .iter()
                .take(3)
                .map(|file| format!("{} ({} lines)", file.path, file.lines))
                .collect(),
        });
    }

    if signals.todo.fixme > 10 {
        issues.push(Issue {
            severity: Severity::Warning,
            category: "code-signals".into(),
            title: format!("High FIXME count ({})", signals.todo.fixme),
            detail: "A high FIXME count may indicate unresolved defects or unstable code.".into(),
            evidence: vec![],
        });
    }

    if structure.duplicate_groups.len() >= 3 {
        issues.push(Issue {
            severity: Severity::Warning,
            category: "duplication".into(),
            title: format!(
                "Duplicate file groups detected ({})",
                structure.duplicate_groups.len()
            ),
            detail: "Identical files were detected by content hash.".into(),
            evidence: structure
                .duplicate_groups
                .iter()
                .take(2)
                .map(|group| group.files.join(", "))
                .collect(),
        });
    }

    if !structure.problems.is_empty() {
        issues.push(Issue {
            severity: Severity::Warning,
            category: "structure".into(),
            title: "Structural problems detected".into(),
            detail: "One or more conventional project-layout checks failed.".into(),
            evidence: structure.problems.iter().take(4).cloned().collect(),
        });
    }

    if repository.present
        && repository.modified_files + repository.staged_files + repository.untracked_files > 20
    {
        issues.push(Issue {
            severity: Severity::Info,
            category: "git".into(),
            title: "Many uncommitted changes".into(),
            detail: "The working tree contains a substantial number of changed or untracked files."
                .into(),
            evidence: vec![],
        });
    }
    issues.sort_by_key(|issue| std::cmp::Reverse(issue.severity));
    issues
}
