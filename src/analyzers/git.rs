use crate::analysis::RepositoryReport;
use std::process::Command;

pub fn analyze(root: &std::path::Path) -> RepositoryReport {
    let mut report = RepositoryReport {
        present: is_repository(root),
        branch: None,
        modified_files: 0,
        staged_files: 0,
        untracked_files: 0,
        ahead: None,
        behind: None,
        last_commit_unix_seconds: None,
        last_commit_message: None,
        last_commit_author: None,
    };
    if !report.present {
        return report;
    }

    report.branch = command_text(root, ["rev-parse", "--abbrev-ref", "HEAD"]);
    if let Some(commit) = command_lines(root, ["log", "-1", "--format=%ct%x1f%s%x1f%an"]) {
        let mut fields = commit.split('\x1f');
        if let Some(seconds) = fields.next().and_then(|value| value.parse::<i64>().ok()) {
            report.last_commit_unix_seconds = Some(seconds);
        }
        report.last_commit_message = fields.next().map(str::to_string);
        report.last_commit_author = fields.next().map(str::to_string);
    }

    if let Some(status) = command_text(root, ["status", "--porcelain=v1", "--branch"]) {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("##") {
                if let Some(ahead) = rest
                    .split_whitespace()
                    .find_map(|part| part.strip_prefix("ahead "))
                {
                    report.ahead = ahead.parse().ok();
                }
                if let Some(behind) = rest
                    .split_whitespace()
                    .find_map(|part| part.strip_prefix("behind "))
                {
                    report.behind = behind.parse().ok();
                }
                continue;
            }
            let bytes = line.as_bytes();
            if line.starts_with("??") {
                report.untracked_files += 1;
                continue;
            }
            if bytes
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'?')
            {
                report.staged_files += 1;
            }
            if bytes
                .get(1)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'?')
                && !line.starts_with("??")
            {
                report.modified_files += 1;
            }
        }
    }
    report
}

fn is_repository(root: &std::path::Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn command_text<const N: usize>(root: &std::path::Path, arguments: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .ok()?;
    (output.status.success()).then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    })
}

fn command_lines<const N: usize>(root: &std::path::Path, arguments: [&str; N]) -> Option<String> {
    command_text(root, arguments)
}
