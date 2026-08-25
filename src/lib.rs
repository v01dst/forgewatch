pub mod analysis;
pub mod analyzers;
pub mod cli;
pub mod error;
pub mod fs;
pub mod report;

#[cfg(test)]
mod tests {
    use crate::analysis::FileRecord;
    use crate::analyzers::languages::language_for_path;
    use std::path::PathBuf;

    #[test]
    fn detects_supported_languages() {
        for (path, expected) in [
            ("src/main.rs", "Rust"),
            ("index.js", "JavaScript"),
            ("app.py", "Python"),
            ("main.go", "Go"),
            ("Main.java", "Java"),
            ("main.c", "C"),
            ("main.cpp", "C++"),
            ("index.php", "PHP"),
            ("app.rb", "Ruby"),
        ] {
            assert_eq!(language_for_path(&PathBuf::from(path)), Some(expected));
        }
        assert_eq!(language_for_path(&PathBuf::from("image.png")), None);
    }

    #[test]
    fn file_record_starts_empty() {
        let record = FileRecord::new(PathBuf::from("a.rs"), 10, false);
        assert_eq!(record.line_count, 0);
        assert!(!record.is_symlink);
    }
}
