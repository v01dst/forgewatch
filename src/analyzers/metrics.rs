use crate::analysis::{AnalysisContext, CodeSignals, CommentStyle, TodoMatch, TodoSummary};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn analyze(context: &AnalysisContext, primary_language: &Option<String>) -> CodeSignals {
    let mut todo = TodoSummary::default();
    let mut large_files = Vec::new();
    let threshold = context.options.large_file_threshold;

    for file in &context.input.files {
        let language = crate::analyzers::languages::language_for_path(&file.relative_path);
        if let Some(language) = language {
            if file.line_count >= threshold {
                large_files.push(crate::analysis::LargeFile {
                    path: file.relative_path.display().to_string(),
                    lines: file.line_count,
                    language: language.to_string(),
                });
            }
        }
        if file.size > context.options.todo_scan_limit {
            continue;
        }
        let style = primary_language.as_deref().and_then(comment_style);
        count_markers(&context.root.join(&file.relative_path), style, &mut todo);
    }

    CodeSignals {
        todo,
        large_files,
        comment_ratio: context
            .input
            .files
            .iter()
            .map(|file| file.comment_lines)
            .sum::<u64>() as f64
            / context
                .input
                .files
                .iter()
                .map(|file| file.line_count)
                .sum::<u64>()
                .max(1) as f64,
        long_lines: context.input.files.iter().map(|file| file.long_lines).sum(),
    }
}

fn count_markers(path: &std::path::Path, style: Option<CommentStyle>, summary: &mut TodoSummary) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    for (line_number, line) in BufReader::new(file).lines().enumerate() {
        let Ok(line) = line else { break };
        let Some(style) = style else { continue };
        if !line.trim_start().starts_with(style.marker()) {
            continue;
        }
        for marker in ["TODO", "FIXME"] {
            if line.contains(marker) {
                if marker == "TODO" {
                    summary.todo += 1;
                } else {
                    summary.fixme += 1;
                }
                summary.total += 1;
                if summary.examples.len() < 10 {
                    summary.examples.push(TodoMatch {
                        path: path.display().to_string(),
                        line: line_number + 1,
                        text: line.trim().chars().take(180).collect(),
                        marker: marker.to_string(),
                    });
                }
            }
        }
    }
}

fn comment_style(language: &str) -> Option<CommentStyle> {
    match language {
        "Rust" | "JavaScript" | "TypeScript" | "Go" | "Java" | "C" | "C++" => {
            Some(CommentStyle::Slash)
        }
        "Python" | "Ruby" | "PHP" => Some(CommentStyle::Hash),
        _ => None,
    }
}
