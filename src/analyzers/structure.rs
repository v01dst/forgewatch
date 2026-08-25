use crate::analysis::{AnalysisContext, DuplicateGroup, LanguageReport, StructureReport};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;

const DUPLICATE_SAMPLE_SIZE: usize = 64 * 1024;

pub fn analyze(context: &AnalysisContext, languages: &LanguageReport) -> StructureReport {
    let mut test_locations: HashSet<String> = HashSet::new();
    let mut has_tests = false;
    let mut source_files_without_test_coverage_estimate = 0u64;

    for file in &context.input.files {
        let path_text = file.relative_path.display().to_string().replace('\\', "/");
        let path_lower = path_text.to_ascii_lowercase();
        if path_lower.contains("/tests/")
            || path_lower.ends_with("_test.go")
            || path_lower.ends_with(".test.js")
            || path_lower.ends_with(".test.tsx")
            || path_lower.ends_with("test.py")
            || path_lower.ends_with("spec.rb")
            || path_lower.starts_with("tests/")
            || path_lower.starts_with("test/")
        {
            has_tests = true;
            test_locations.insert(path_text.clone());
        } else if crate::analyzers::languages::language_for_path(&file.relative_path).is_some() {
            source_files_without_test_coverage_estimate += 1;
        }
    }
    if has_tests {
        source_files_without_test_coverage_estimate =
            source_files_without_test_coverage_estimate.saturating_sub(test_locations.len() as u64);
    }

    let duplicate_groups = find_duplicates(context);
    let mut problems = Vec::new();
    if context
        .input
        .files
        .iter()
        .any(|f| f.size > 50 * 1024 * 1024)
    {
        problems.push("Very large checked-in files detected (>50 MiB)".to_string());
    }
    if has_manifest_and_no_readme(context) {
        problems.push("Dependency manifest present without README".to_string());
    }
    if languages.primary.is_none() && !context.input.files.is_empty() {
        problems.push("No recognized programming-language files found".to_string());
    }

    StructureReport {
        has_tests,
        test_locations: test_locations.into_iter().collect(),
        source_files_without_test_coverage_estimate: Some(
            source_files_without_test_coverage_estimate,
        ),
        duplicate_groups,
        problems,
    }
}

fn has_manifest_and_no_readme(context: &AnalysisContext) -> bool {
    let has_manifest = context.input.files.iter().any(|f| {
        matches!(
            f.relative_path
                .file_name()
                .and_then(std::ffi::OsStr::to_str),
            Some("Cargo.toml" | "package.json" | "requirements.txt" | "go.mod")
        )
    });
    let has_readme = context.input.files.iter().any(|f| matches!(f.relative_path.file_name().and_then(std::ffi::OsStr::to_str).map(str::to_ascii_lowercase).as_deref(), Some(name) if name.starts_with("readme.")));
    has_manifest && !has_readme
}

fn find_duplicates(context: &AnalysisContext) -> Vec<DuplicateGroup> {
    let mut candidates: HashMap<u64, Vec<&crate::analysis::FileRecord>> = HashMap::new();
    for file in &context.input.files {
        if file.size < 128 || file.size > 10 * 1024 * 1024 {
            continue;
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write_u64(file.size);
        candidates.entry(hasher.finish()).or_default().push(file);
    }

    let mut hashes: HashMap<(u64, String), Vec<String>> = HashMap::new();
    for (_, group) in candidates {
        if group.len() < 2 {
            continue;
        }
        for file in group {
            let path = context.root.join(&file.relative_path);
            if let Some(digest) = sample_hash(&path, file.size) {
                hashes
                    .entry((digest, file.size.to_string()))
                    .or_default()
                    .push(file.relative_path.display().to_string());
            }
        }
    }

    let mut groups: Vec<_> = hashes
        .into_values()
        .filter(|files| files.len() > 1)
        .map(DuplicateGroup::from_files)
        .collect();
    groups.sort_by(|left, right| {
        right
            .files
            .len()
            .cmp(&left.files.len())
            .then(left.files.first().cmp(&right.files.first()))
    });
    groups.truncate(25);
    groups
}

fn sample_hash(path: &std::path::Path, size: u64) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let length = DUPLICATE_SAMPLE_SIZE.min(size as usize);
    let mut buffer = vec![0; length];
    file.read_exact(&mut buffer).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(&buffer);
    hasher.write_u64(size);
    Some(hasher.finish())
}

impl DuplicateGroup {
    fn from_files(files: Vec<String>) -> Self {
        Self {
            files,
            size_bytes: 0,
        }
    }
}
