use crate::analysis::ProjectHealthReport;
use anyhow::Result;

const WIDTH: usize = 78;

pub fn render(report: &ProjectHealthReport, no_color: bool, verbose: bool) -> Result<()> {
    let color = !no_color && supports_color();
    let mut out = String::new();
    out.push_str(&heading("FORGEWATCH", color));
    out.push_str(&format!("Project: {}\n", report.project_name));
    out.push_str(&"─".repeat(WIDTH));
    out.push('\n');

    out.push_str(&section("Languages"));
    if report.languages.languages.is_empty() {
        out.push_str("No recognized source files\n");
    } else {
        let width = report
            .languages
            .languages
            .iter()
            .map(|l| l.name.len())
            .max()
            .unwrap_or(0);
        for language in &report.languages.languages {
            out.push_str(&format!(
                "{:<width$} {:>5.1}%  ({} lines)\n",
                language.name,
                language.percentage,
                thousands(language.lines),
                width = width
            ));
        }
    }

    out.push_str(&section("Files"));
    out.push_str(&format!(
        "Files        {}\n",
        thousands(report.scan.files_scanned)
    ));
    out.push_str(&format!(
        "Source lines {}\n",
        thousands(report.languages.total_source_lines)
    ));
    out.push_str(&format!(
        "Scanned files {}\n",
        report.structure.test_locations.len()
    ));

    out.push_str(&section("Dependencies"));
    if report.dependencies.manifests.is_empty() {
        out.push_str("No dependency manifests found\n");
    } else {
        out.push_str(&format!(
            "Total       {}\n",
            thousands(report.dependencies.total)
        ));
        out.push_str(&format!(
            "Potentially outdated {}\n",
            thousands(report.dependencies.potentially_outdated)
        ));
        out.push_str(&format!(
            "Local or Git references {}\n",
            thousands(report.dependencies.local_or_git)
        ));
    }

    out.push_str(&section("Code Signals"));
    out.push_str(&format!(
        "TODO   {}\n",
        thousands(report.code_signals.todo.todo)
    ));
    out.push_str(&format!(
        "FIXME  {}\n",
        thousands(report.code_signals.todo.fixme)
    ));
    out.push_str(&format!(
        "Large files {}\n",
        thousands(report.code_signals.large_files.len() as u64)
    ));
    out.push_str(&format!(
        "Long lines {}\n",
        thousands(report.code_signals.long_lines)
    ));

    out.push_str(&section("Git"));
    render_git(&mut out, report);

    out.push_str(&section("Potential Issues"));
    if report.issues.is_empty() {
        out.push_str("No issues detected by current checks\n");
    } else {
        for issue in &report.issues {
            out.push_str(&format!("{} {}\n", symbol(issue.severity), issue.title));
        }
    }

    out.push_str(&section("Health Score"));
    out.push_str(&health_bar(report.health_score.score));
    out.push_str(&format!(
        " {} / 100 ({})\n",
        report.health_score.score, report.health_score.grade
    ));

    if verbose {
        out.push_str(&section("Partial Analysis"));
        for item in &report.partial_analysis {
            out.push_str(&format!("- {item}\n"));
        }
        out.push_str(&section("Manifests"));
        if report.dependencies.manifests.is_empty() {
            out.push_str("None\n");
        }
        for manifest in &report.dependencies.manifests {
            out.push_str(&format!(
                "- {} [{}] total={} unpinned={} local/git={}\n",
                manifest.path,
                manifest.ecosystem,
                manifest.total,
                manifest.unpinned,
                manifest.local_or_git
            ));
        }
        if !report.structure.duplicate_groups.is_empty() {
            out.push_str(&section("Duplicate Groups"));
            for group in report.structure.duplicate_groups.iter().take(10) {
                out.push_str(&format!("- {}\n", group.files.join(" | ")));
            }
        }
    }
    print!("{out}");
    Ok(())
}

fn render_git(out: &mut String, report: &ProjectHealthReport) {
    if !report.git.present {
        out.push_str("Not a Git repository\n");
        return;
    }
    out.push_str(&format!(
        "Branch      {}\n",
        report.git.branch.as_deref().unwrap_or("(detached)")
    ));
    out.push_str(&format!(
        "Modified    {} files\n",
        report.git.modified_files
    ));
    out.push_str(&format!("Staged      {} files\n", report.git.staged_files));
    out.push_str(&format!(
        "Untracked   {} files\n",
        report.git.untracked_files
    ));
    if let Some(seconds) = report.git.last_commit_unix_seconds {
        out.push_str(&format!("Last commit {}\n", human_age(seconds)));
    }
}

fn section(title: &str) -> String {
    format!("\n{}\n", title)
}

fn heading(text: &str, color: bool) -> String {
    if color {
        format!("\x1b[1;36m{text}\x1b[0m\n")
    } else {
        format!("{text}\n")
    }
}

fn health_bar(score: u8) -> String {
    const CELLS: usize = 24;
    let filled = (usize::from(score) * CELLS) / 100;
    format!("{}{}", "█".repeat(filled), "░".repeat(CELLS - filled))
}

fn symbol(severity: crate::analysis::Severity) -> &'static str {
    match severity {
        crate::analysis::Severity::Critical => "✖",
        crate::analysis::Severity::Warning => "⚠",
        crate::analysis::Severity::Info => "ℹ",
    }
}

fn thousands(value: u64) -> String {
    let text = value.to_string();
    let mut result = String::new();
    for (index, character) in text.chars().enumerate() {
        if index > 0 && (text.len() - index) % 3 == 0 {
            result.push(',');
        }
        result.push(character);
    }
    result
}

fn human_age(unix_seconds: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let age = (now - unix_seconds).max(0);
    let label = if age < 60 {
        format!("{age} seconds")
    } else if age < 3600 {
        format!("{} minutes", age / 60)
    } else if age < 86_400 {
        format!("{} hours", age / 3600)
    } else {
        format!("{} days", age / 86_400)
    };
    format!("{label} ago")
}

fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::IsTerminal::is_terminal(&std::io::stdout())
}
