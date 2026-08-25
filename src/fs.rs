use crate::analysis::{AnalysisInput, FileRecord};
use crate::error::Result;
use ignore::{WalkBuilder, WalkState};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::path::Path;
use std::sync::mpsc;

const MAX_INSPECTED_FILE_SIZE: u64 = 8 * 1024 * 1024;
const MAX_LINE_LENGTH: usize = 4_000;

pub fn collect_input(root: &Path, max_file_size: u64) -> Result<AnalysisInput> {
    let (sender, receiver) = mpsc::channel::<FileRecord>();
    let effective_limit = max_file_size.min(MAX_INSPECTED_FILE_SIZE);
    let walker_root = root.to_path_buf();

    let closure_root = walker_root.clone();
    let prefix_root = walker_root.clone();
    WalkBuilder::new(&walker_root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .parents(true)
        .require_git(false)
        .filter_entry(|entry| !is_special_file(entry.path()))
        .build_parallel()
        .run(move || {
            let sender = sender.clone();
            Box::new({
                let closure_root = closure_root.clone();
                let prefix_root = prefix_root.clone();
                move |entry| {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => return WalkState::Continue,
                    };
                    let path = entry.path();
                    if path == closure_root || !entry.file_type().is_some_and(|kind| kind.is_file())
                    {
                        return WalkState::Continue;
                    }
                    if is_special_file(path) {
                        return WalkState::Continue;
                    }
                    let metadata = match entry.metadata() {
                        Ok(metadata) => metadata,
                        Err(_) => return WalkState::Continue,
                    };
                    let size = metadata.len();
                    let relative = path
                        .strip_prefix(&prefix_root)
                        .unwrap_or(path)
                        .to_path_buf();
                    let mut record = FileRecord::new(relative, size, metadata.is_symlink());

                    if record.size <= effective_limit && looks_like_text(path) {
                        analyze_file(path, &mut record);
                    }
                    if sender.send(record).is_err() {
                        return WalkState::Quit;
                    }
                    WalkState::Continue
                }
            })
        });

    let mut files: Vec<_> = receiver.into_iter().collect();
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(AnalysisInput::new(walker_root, files))
}

fn analyze_file(path: &Path, record: &mut FileRecord) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let reader = BufReader::new(file);
    for line in reader.lines().map_while(std::result::Result::ok) {
        record.line_count += 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            record.comment_lines += 1;
        }
        if line.len() > MAX_LINE_LENGTH {
            record.long_lines += 1;
        }
    }
}

fn looks_like_text(path: &Path) -> bool {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut buffer = [0; 8_192];
    let mut handle = file.take(buffer.len() as u64);
    if handle.read_exact(&mut buffer).is_ok() {
        return false;
    }
    let read = handle.stream_position().unwrap_or(0) as usize;
    buffer[..read].iter().all(|byte| *byte != 0)
}

fn is_special_file(path: &Path) -> bool {
    const IGNORED_DIRECTORIES: [&str; 9] = [
        ".git",
        ".hg",
        ".svn",
        "node_modules",
        "vendor",
        "target",
        "dist",
        "build",
        "__pycache__",
    ];
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(name) if IGNORED_DIRECTORIES.contains(&name)
        )
    })
}
