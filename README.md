# ForgeWatch

ForgeWatch is a fast, local-first CLI for inspecting the technical health of software projects. It scans a directory, respects ignore rules, summarizes languages and dependencies, inspects Git state, detects maintainability signals, and produces a transparent health score.

ForgeWatch performs practical static inspection rather than pretending to provide deep semantic analysis. Registry access is never required, dependency versions are not downloaded, and inferred checks are explicitly labeled.

## Features

- Automatic project-type detection from files and manifests.
- Parallel recursive scanning with `.gitignore`, global excludes, and `.git/info/exclude` support.
- Line, comment-ratio, TODO/FIXME, long-line, and oversized-file signals.
- Manifest-aware analysis for Cargo, npm, Python requirements, Go modules, Maven, and RubyGems.
- Git branch, status, ahead/behind, author, timestamp, and last-commit summary.
- Duplicate-file detection using bounded content sampling.
- Stable JSON output and accessible terminal output with color disabled automatically when unsupported.
- Health score with issue-derived penalties and a letter grade.

## Installation

Rust 1.75 or newer is required.

```bash
cargo install --path .
```

For development:

```bash
cargo build
```

The release binary is written to `target/release/forgewatch`.

## Usage

```bash
forgewatch .
forgewatch /path/to/project
forgewatch --json .
forgewatch --no-color --verbose .
forgewatch --help
forgewatch --version
```

### Options

| Option | Description |
|---|---|
| `<PATH>` | Project directory to analyze. |
| `--json` | Print stable machine-readable JSON. |
| `--format <terminal\|json>` | Select the renderer explicitly. |
| `--no-color` | Disable terminal styling. |
| `--max-file-size <KiB>` | Content-inspection size limit; default is 8192 KiB. |
| `--verbose` | Show manifests, partial-analysis notes, and duplicate groups. |

## Supported Languages

Rust, JavaScript/JSX, TypeScript/TSX, Python, Go, Java, C, C++, PHP, and Ruby are recognized. Detection is extension-based and reported as such in the JSON `partial_analysis` field.

Dependency support is manifest-only: Cargo, npm/package.json, requirements.txt, go.mod, Maven pom.xml, and Gemfile. Unsupported manifests are ignored rather than guessed at.

## Architecture

```text
src/
  main.rs                 Binary entry point
  lib.rs                  Library root
  cli.rs                  Argument parsing
  error.rs                Typed errors
  fs.rs                   Ignore-aware parallel scanning
  analysis.rs             Shared models, context creation, orchestration input
  analyzers/
    dependencies.rs       Manifest parsing
    git.rs                Git CLI integration
    issues.rs             Issue detection
    languages.rs          Language classification
    metrics.rs            Source metrics
    score.rs              Health scoring
    structure.rs          Tests, layout, duplication
  report/
    json.rs               Stable JSON rendering
    terminal.rs           Human terminal report
```

## JSON Output

`schema_version` is currently `1`. The top-level object contains:

```json
{
  "schema_version": 1,
  "project_name": "example",
  "analyzed_path": "/absolute/project/path",
  "generated_at_unix_seconds": 1760000000,
  "scan": {},
  "languages": {},
  "dependencies": {},
  "git": {},
  "code_signals": {},
  "structure": {},
  "issues": [],
  "health_score": {},
  "partial_analysis": []
}
```

Run `forgewatch --json tests/fixtures/rust_project` to see all fields. Additive fields may be introduced without increasing the schema version. Existing fields will not be renamed or removed within schema version 1.

Errors use this shape on stderr and exit with a nonzero status:

```json
{"error":{"message":"..."}}
```

## Development

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Fixtures for Rust, TypeScript, Python, and Go live under `tests/fixtures/`.

## Reliability Notes

- Unreadable or malformed files are skipped rather than aborting the scan.
- Binary files are excluded from line counting by a NUL-byte probe.
- Symlinks are recorded but not followed into cycles.
- Common build and VCS directories are skipped.
- Projects without Git or dependency manifests receive valid empty reports.

## License

Dual licensed under MIT OR Apache-2.0.
