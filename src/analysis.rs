use crate::error::{ForgeWatchError, Result};
use crate::fs;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AnalysisArguments {
    pub path: PathBuf,
    pub json: bool,
    pub max_file_size: u64,
}

#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub large_file_threshold: u64,
    pub todo_scan_limit: u64,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            large_file_threshold: 400,
            todo_scan_limit: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub relative_path: PathBuf,
    pub size: u64,
    pub is_symlink: bool,
    pub line_count: u64,
    pub comment_lines: u64,
    pub long_lines: u64,
}

impl FileRecord {
    pub fn new(relative_path: PathBuf, size: u64, is_symlink: bool) -> Self {
        Self {
            relative_path,
            size,
            is_symlink,
            line_count: 0,
            comment_lines: 0,
            long_lines: 0,
        }
    }
}

#[derive(Debug)]
pub struct AnalysisInput {
    pub root: PathBuf,
    pub files: Vec<FileRecord>,
}

impl AnalysisInput {
    pub fn new(root: PathBuf, files: Vec<FileRecord>) -> Self {
        Self { root, files }
    }
}

#[derive(Debug)]
pub struct AnalysisContext {
    pub root: PathBuf,
    pub input: AnalysisInput,
    pub options: AnalysisOptions,
    pub partial_analysis: Vec<String>,
}

impl AnalysisContext {
    pub fn project_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
            .to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageUsage {
    pub name: String,
    pub lines: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanSummary {
    pub files_scanned: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    pub languages: Vec<LanguageUsage>,
    pub total_source_lines: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyManifest {
    pub ecosystem: String,
    pub path: String,
    pub total: u64,
    pub direct: Option<u64>,
    pub development: Option<u64>,
    pub locked: Option<u64>,
    pub local_or_git: u64,
    pub unpinned: u64,
    pub parse_status: ParseStatus,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Parsed,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyReportData {
    pub total: u64,
    pub manifests: Vec<DependencyManifest>,
    pub potentially_outdated: u64,
    pub local_or_git: u64,
    pub analysis_level: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryReport {
    pub present: bool,
    pub branch: Option<String>,
    pub modified_files: u64,
    pub staged_files: u64,
    pub untracked_files: u64,
    pub ahead: Option<u64>,
    pub behind: Option<u64>,
    pub last_commit_unix_seconds: Option<i64>,
    pub last_commit_message: Option<String>,
    pub last_commit_author: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentStyle {
    Slash,
    Hash,
}

impl CommentStyle {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Slash => "//",
            Self::Hash => "#",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TodoMatch {
    pub path: String,
    pub line: usize,
    pub marker: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TodoSummary {
    pub total: u64,
    pub todo: u64,
    pub fixme: u64,
    pub examples: Vec<TodoMatch>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LargeFile {
    pub path: String,
    pub lines: u64,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeSignals {
    pub todo: TodoSummary,
    pub large_files: Vec<LargeFile>,
    pub comment_ratio: f64,
    pub long_lines: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructureReport {
    pub has_tests: bool,
    pub test_locations: Vec<String>,
    pub source_files_without_test_coverage_estimate: Option<u64>,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub files: Vec<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthScore {
    pub score: u8,
    pub grade: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectHealthReport {
    pub schema_version: u32,
    pub project_name: String,
    pub analyzed_path: String,
    pub generated_at_unix_seconds: i64,
    pub scan: ScanSummary,
    pub languages: LanguageReport,
    pub dependencies: DependencyReportData,
    pub git: RepositoryReport,
    pub code_signals: CodeSignals,
    pub structure: StructureReport,
    pub issues: Vec<Issue>,
    pub health_score: HealthScore,
    pub partial_analysis: Vec<String>,
}

// Kept as the internal aggregate returned by analyzer orchestration.
pub struct AggregateReport {
    pub project_name: String,
    pub scan: ScanSummary,
    pub languages: LanguageReport,
    pub dependencies: DependencyReportData,
    pub repository: RepositoryReport,
    pub signals: CodeSignals,
    pub structure: StructureReport,
    pub issues: Vec<Issue>,
    pub health_score: HealthScore,
    pub partial_analysis: Vec<String>,
}

pub fn create_context(arguments: &AnalysisArguments) -> Result<AnalysisContext> {
    let root = arguments
        .path
        .canonicalize()
        .map_err(|error| ForgeWatchError::Access(arguments.path.clone(), error))?;
    if !root.is_dir() {
        return Err(ForgeWatchError::NotADirectory(arguments.path.clone()));
    }
    let input = fs::collect_input(&root, arguments.max_file_size)?;
    let mut partial_analysis = vec![
        "language detection uses file extensions".to_string(),
        "dependency checks are manifest-based and do not access registries".to_string(),
        "test coverage is inferred from filenames and conventional directories".to_string(),
    ];
    if input.files.is_empty() {
        partial_analysis.push("no readable source files were found".to_string());
    }
    Ok(AnalysisContext {
        root,
        input,
        options: AnalysisOptions::default(),
        partial_analysis,
    })
}
