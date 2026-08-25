use crate::analysis::AnalysisContext;
use crate::analysis::{DependencyManifest, DependencyReportData, ParseStatus};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn analyze(context: &AnalysisContext) -> DependencyReportData {
    let mut manifests = Vec::new();
    for file in &context.input.files {
        let name = file.relative_path.file_name().and_then(|n| n.to_str());
        let manifest = match name {
            Some("Cargo.toml") => cargo_manifest(&context.root.join(&file.relative_path)),
            Some("package.json") => package_manifest(&context.root.join(&file.relative_path)),
            Some("requirements.txt") => {
                requirements_manifest(&context.root.join(&file.relative_path))
            }
            Some("go.mod") => go_manifest(&context.root.join(&file.relative_path)),
            Some("pom.xml") => maven_manifest(&context.root.join(&file.relative_path)),
            Some("Gemfile") => gemfile_manifest(&context.root.join(&file.relative_path)),
            _ => continue,
        };
        manifests.push(manifest);
    }
    manifests.sort_by(|left, right| left.path.cmp(&right.path));
    let total: u64 = manifests.iter().map(|m| m.total).sum();
    let potentially_outdated: u64 = manifests.iter().map(|m| m.unpinned + m.local_or_git).sum();
    DependencyReportData {
        total,
        potentially_outdated,
        local_or_git: manifests.iter().map(|m| m.local_or_git).sum(),
        manifests,
        analysis_level: "manifest-only".to_string(),
    }
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn base(ecosystem: &str, path: &Path, status: ParseStatus) -> (String, DependencyManifest) {
    (
        path.display().to_string(),
        DependencyManifest {
            ecosystem: ecosystem.to_string(),
            path: String::new(),
            total: 0,
            direct: None,
            development: None,
            locked: None,
            local_or_git: 0,
            unpinned: 0,
            parse_status: status,
        },
    )
}

fn cargo_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("Cargo", path);
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        return failed("Cargo", path);
    };
    let mut manifest = parsed("Cargo", path);
    manifest.direct = count_table(value.get("dependencies"));
    manifest.development = count_table(value.get("dev-dependencies"));
    manifest.total = manifest.direct.unwrap_or_default() + manifest.development.unwrap_or_default();
    for section in ["dependencies", "dev-dependencies"] {
        if let Some(table) = value.get(section).and_then(toml::Value::as_table) {
            manifest.local_or_git +=
                table.values().filter(|value| is_local_toml(value)).count() as u64;
        }
    }
    manifest
}

fn package_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("npm", path);
    };
    let Ok(value) = serde_json::from_str::<Value>(&content) else {
        return failed("npm", path);
    };
    let mut manifest = parsed("npm", path);
    manifest.direct = count_object(value.get("dependencies"));
    manifest.development = count_object(value.get("devDependencies"));
    manifest.locked = value
        .pointer("/packageLock/dependencies")
        .and_then(Value::as_array)
        .map(|a| a.len() as u64);
    manifest.total = manifest.direct.unwrap_or_default() + manifest.development.unwrap_or_default();
    for key in ["dependencies", "devDependencies"] {
        if let Some(object) = value.get(key).and_then(Value::as_object) {
            manifest.unpinned += object
                .values()
                .filter(|v| v.as_str().is_some_and(|s| s == "*" || s == "latest"))
                .count() as u64;
            manifest.local_or_git += object
                .values()
                .filter(|v| v.as_str().is_some_and(is_url_like))
                .count() as u64;
        }
    }
    manifest
}

fn requirements_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("Python", path);
    };
    let lines: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let mut manifest = parsed("Python", path);
    manifest.total = lines.len() as u64;
    manifest.direct = Some(manifest.total);
    manifest.unpinned = lines
        .iter()
        .filter(|line| !line.contains("==") && !line.contains("@"))
        .count() as u64;
    manifest.local_or_git = lines
        .iter()
        .filter(|line| line.contains("git+") || line.contains("-e "))
        .count() as u64;
    manifest
}

fn go_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("Go", path);
    };
    let require_count = content.matches("require ").count() as u64;
    let mut manifest = parsed("Go", path);
    manifest.total = require_count
        + content
            .lines()
            .filter(|line| line.trim_start().starts_with("require ") && line.contains(' '))
            .count() as u64;
    manifest.direct = Some(manifest.total);
    manifest.local_or_git = content
        .lines()
        .filter(|line| line.contains("git::") || line.contains("./"))
        .count() as u64;
    manifest.parse_status = ParseStatus::Partial;
    manifest
}

fn maven_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("Maven", path);
    };
    let mut manifest = partial("Maven", path);
    manifest.total = content.matches("<dependency>").count() as u64;
    manifest.direct = Some(manifest.total);
    manifest
}

fn gemfile_manifest(path: &Path) -> DependencyManifest {
    let Some(content) = read(path) else {
        return failed("RubyGems", path);
    };
    let mut manifest = partial("RubyGems", path);
    manifest.total = content
        .lines()
        .filter(|line| line.trim_start().starts_with("gem "))
        .count() as u64;
    manifest.direct = Some(manifest.total);
    manifest.local_or_git = content
        .lines()
        .filter(|line| line.contains("git:") || line.contains("path:"))
        .count() as u64;
    manifest
}

fn parsed(ecosystem: &str, path: &Path) -> DependencyManifest {
    let (_, mut manifest) = base(ecosystem, path, ParseStatus::Parsed);
    manifest.path = path
        .strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(path)
        .display()
        .to_string();
    manifest
}

fn partial(ecosystem: &str, path: &Path) -> DependencyManifest {
    let (relative, mut manifest) = base(ecosystem, path, ParseStatus::Partial);
    manifest.path = relative;
    manifest
}

fn failed(ecosystem: &str, path: &Path) -> DependencyManifest {
    let (_, mut manifest) = base(ecosystem, path, ParseStatus::Failed);
    manifest.path = path.display().to_string();
    manifest
}

fn count_table(value: Option<&toml::Value>) -> Option<u64> {
    value
        .and_then(toml::Value::as_table)
        .map(|table| table.len() as u64)
}

fn count_object(value: Option<&Value>) -> Option<u64> {
    value
        .and_then(Value::as_object)
        .map(|object| object.len() as u64)
}

fn is_local_toml(value: &toml::Value) -> bool {
    value
        .as_table()
        .is_some_and(|table| table.contains_key("git") || table.contains_key("path"))
}

fn is_url_like(value: &str) -> bool {
    value.contains("://") || value.starts_with("git@") || value.starts_with("file:")
}
