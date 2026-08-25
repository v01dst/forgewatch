use crate::analysis::{AnalysisContext, LanguageReport, LanguageUsage};
use std::collections::BTreeMap;

pub fn analyze(context: &AnalysisContext) -> LanguageReport {
    let mut totals: BTreeMap<&'static str, u64> = BTreeMap::new();
    for file in &context.input.files {
        if let Some(language) = language_for_path(&file.relative_path) {
            *totals.entry(language).or_default() += file.line_count.max(1);
        }
    }
    let total: u64 = totals.values().sum();
    let mut languages: Vec<LanguageUsage> = totals
        .into_iter()
        .map(|(language, lines)| LanguageUsage {
            name: language.to_string(),
            lines,
            percentage: if total == 0 {
                0.0
            } else {
                lines as f64 * 100.0 / total as f64
            },
        })
        .collect();
    languages.sort_by(|left, right| {
        right
            .lines
            .cmp(&left.lines)
            .then(left.name.cmp(&right.name))
    });

    let primary = languages.first().map(|language| language.name.clone());
    LanguageReport {
        primary,
        languages,
        total_source_lines: total,
    }
}

pub fn language_for_path(path: &std::path::Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    let result = match (extension.as_str(), name.as_str()) {
        ("rs", _) | ("toml", "cargo.toml") => "Rust",
        ("js" | "cjs" | "mjs", _)
        | ("json", "package.json")
        | ("json", "package-lock.json")
        | ("json", "npm-shrinkwrap.json") => "JavaScript",
        ("jsx", _) => "JSX",
        ("ts", _) => "TypeScript",
        ("tsx", _) => "TSX",
        ("py", _) | ("pyi", _) => "Python",
        ("go", _) => "Go",
        ("java", _) => "Java",
        ("c", _) | ("h", _) => "C",
        ("cc" | "cpp" | "cxx" | "hpp" | "hh", _) => "C++",
        ("php", _) => "PHP",
        ("rb", _) | ("erb", _) => "Ruby",
        _ => return None,
    };
    Some(result)
}
